// #![deny(unused_mut)]
extern crate zookeeper;
extern crate uuid;
extern crate crossbeam;

// use std::String;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::{Sender,Receiver};
use std::sync::atomic::{
    AtomicBool,
    Ordering::SeqCst,
};
use self::uuid::Uuid;
use self::zookeeper::{Acl, CreateMode, WatchedEvent, Watcher, ZooKeeper,ZkResult};
use self::zookeeper::ZooKeeperExt;
use errors::RcpError;
// use self::crossbeam;


/// try implement idiom of lock with acquire and release
/// yet, not so successful
struct Lock{
    /// use Mutex to ensure thread safety
    //lock: Arc<Mutex<InnerLock>>,
    lock: Mutex<ZkDistrLock>,
}

/// protect code section in multi thread situation
struct ZkCriticalSection {
    lock : ZkDistrLock,
}

/// todo: critical section for ReadWrite Lock
struct ReadCriticalSection{
    lock : ZkDistrLock,
}
struct WirteCriticalSection{
    lock : ZkDistrLock,
}

// got this idea form kazoo
enum LockType{
    Lock,
    ReadLock,
    WriteLock,
}

//trait LockType2{
//    fn get_name(&self) -> (String, Vec<String>);
//}
//
//struct MutexType;
//struct ReadType;
//struct WriteType;
//
//impl LockType2 for MutexType{
//    fn get_name(&self) { ("__lock__", vec![Strint("__lock__"),Strint("__rlock__")] ) }
//}
//
//impl LockType2 for ReadType{
//    fn get_name(&self) { ("__rlock__", vec![Strint("__lock__")] ) }
//}
//
//impl LockType2 for WriteType{
//    fn get_name(&self) { ("__lock__", vec![Strint("__lock__"),Strint("__rlock__")] ) }
//}

struct ZkDistrLock {
    zk: Arc<Box<ZooKeeper>>,
    is_acquired : bool,
    path : String,
    identifier : String,
    node_name : String,
    exclude_names: Vec<String>,
    ensure_path : bool,
    prefix:String,
    create_path:String,
    node:String,
    watch_tx : Sender<()>,
    change_rx : Receiver<()>,
    // canceled: bool,
    canceled: Arc<AtomicBool>,
}

impl ZkDistrLock {
    pub fn new(zk: Arc<Box<ZooKeeper>>, path : String, identifier : String, lock_type :LockType ) -> ZkDistrLock {
        let (node_name, exclude_names)= match lock_type {
            LockType::Lock => ("__lock__", vec!["__lock__".to_string(),"__rlock__".to_string()] ),
            LockType::ReadLock => ("__rlock__", vec!["__lock__".to_string()] ),
            LockType::WriteLock => ("__lock__", vec!["__lock__".to_string(),"__rlock__".to_string()] ),
        };

        let uid = Uuid::new_v4();
        let prefix = uid.to_string()+node_name;

        let create_path = format!("{}/{}",&path, &prefix );

        let (tx, rx) = mpsc::channel();

        let mut lock = ZkDistrLock {
            zk: zk,
            is_acquired : false,
            path: path,
            identifier : identifier,
            node_name: node_name.to_string().clone(),
            exclude_names: exclude_names,
            ensure_path: false,
            prefix : prefix.clone(),
            create_path: create_path,
            node:"".to_string().clone(),
            watch_tx: tx,
            change_rx: rx,
            canceled: Arc::new(AtomicBool::new(false)),
        };

        lock
    }

    pub fn acquire(&mut self)->Result<bool, RcpError>{

        match self.zk.exists(&self.path, false){
            Ok(o)=> match o {
                None => self.zk.ensure_path(&self.path).unwrap(),
                _ => ()
            },
            Err(e) => return Err(RcpError::ZkErr(e)),
        };

        let node_path = self.zk.create(&self.create_path, self.identifier.as_bytes().to_vec(),
                            Acl::open_unsafe().clone(),
                            CreateMode::EphemeralSequential)?;

        self.node = (&node_path)[self.path.len() + 1..].to_string();

        // println!("node {}",&self.node);


        loop{
            let is_first = self.is_first()?;
            // println!("is first {}", is_first);
            if is_first {
                self.is_acquired = true;
                return  Ok(true)
            }

            let pre = self.predecessor()?;
            // println!("predecessor {}", pre);

            // self.zk.get_data_w(&pre, watch_change );
            // self.zk.get_data_w(&pre, |event| self.watch_change(event) );
            let watch_tx = self.watch_tx.clone();
            self.zk.get_data_w(&format!("{}/{}",self.path,&pre), move |event| watch_tx.send(()).unwrap() );
            self.wait_for_change();

            if self.canceled.load(SeqCst) {
                return Ok(false)
            }
        }
    }

//    fn watch_change(&self, event : WatchedEvent) {
//        self.watch_tx.send(()).unwrap();
//    }

    fn wait_for_change(&self){
        // println!("wait_for_change");
        self.change_rx.recv().unwrap();
        // println!("changed");
    }

    fn get_sorted_children(&self)->Result<Vec<String>, RcpError>{
        let mut children = self.zk.get_children(&self.path, false)?;
        children.sort_by_key( |x|{
            for name in vec!["__lock__","__rlock__"]{
                match x.find(&name){
                    Some(pos) => {
                        // println!("key {}",&x[pos+name.len()..]);
                        return (&x[pos+name.len()..]).to_string();
                        },
                    None => "~",
                };
            };
            "~".to_string()
        });
        Ok(children)
    }

    fn is_first(&self)->Result<bool, RcpError>{
        let children = self.get_sorted_children()?;
        // println!("{:?}",&children);

        if children.len() < 1 {
            return Err(RcpError::InternalError);
        }

        if children[0]== self.node {
            // self be first
            return Ok(true)
        }
        else {
            return Ok(false)
        }
    }

    fn predecessor(&self)->Result<String, RcpError> {
        let children = self.get_sorted_children()?;
        let pos = match &children.iter().position(| ref x| *x == &self.node) {
            Some(pos) => pos.clone(),
            None => {// println!("bug happen");
                return Err(RcpError::InternalError)},
        };
        // println!("debug pos {}",pos);

        Ok(children[pos -1].clone())
    }

    pub fn release(&mut self)->Result<bool, RcpError>{
        if ! &self.is_acquired {
            return Ok(false);
        }

        self.zk.delete(&format!("{}/{}",self.path,self.node), None)?;

        self.is_acquired = false;
        self.node = "".to_string();

        Ok(true)
    }

//    pub fn cancel(&self) ->Box<Fn()> {
//
//        let canceled = self.canceled.clone();
//        let watch_tx = self.watch_tx.clone();
//
//        let cancel_closure= move | |{
//            canceled.store(true, SeqCst);
//            watch_tx.send(()).unwrap();
//        };
//
//        Box::new(cancel_closure)
//    }
}

impl Lock {
    pub fn new(zk: Arc<Box<ZooKeeper>>, path : String, identifier : String, lock_type :LockType ) -> Lock{
        let mut lock = Lock{
            // lock: Arc::new(Mutex::new(InnerLock::new(zk,path,identifier,lock_type))),
            lock: Mutex::new(ZkDistrLock::new(zk, path, identifier, lock_type)),
        };

        lock
    }

    pub fn acquire(&self)->Result<bool, RcpError>{
        self.lock
            .lock()
            //.unwrap()
            .unwrap_or_else(|e| e.into_inner())
            .acquire()
    }

    pub fn release(&self)->Result<bool, RcpError>{
        self.lock
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .release()
    }
}

impl ZkCriticalSection {
    pub fn new( zk: Arc<Box<ZooKeeper>>, path : String, identifier : String)->Arc<Mutex<ZkCriticalSection>>{
        let cs = Arc::new(Mutex::new(
            ZkCriticalSection {lock: ZkDistrLock::new(zk, path, identifier, LockType::Lock)}));
        cs
    }

    pub fn guard<F:Fn()>(&mut self, fun: F)->Result<(), RcpError>{
        self.lock.acquire()?;
        fun();
        self.lock.release()?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    extern crate core;

    use super::*;
    use std::time::{Duration,Instant};
    use std::time;
    use std::thread;
    use std::sync;
    use std::cell::Cell;
    use self::core::sync::atomic;
    use self::core::sync::atomic::Ordering;

    struct LoggingWatcher;
    impl Watcher for LoggingWatcher {
        fn handle(&self, e: WatchedEvent) {
            println!("{:?}", e)
        }
    }

    struct DummyWatcher;
    impl Watcher for DummyWatcher {
        fn handle(&self, e: WatchedEvent) {
            ()
        }
    }

    fn connect<W>( watcher: W)
        where W: Watcher + 'static
    {
        let w = watcher;
//        let () = w;
    }

    #[test]
    fn test_internal_lock() {

        let zkserver= "127.0.0.1:2181";

        let dummpy_watcher = sync::Arc::new(Box::new(DummyWatcher));
        connect(DummyWatcher);

        let zk = ZooKeeper::connect(&zkserver, Duration::from_secs(15), DummyWatcher).expect("debug 1");
        let zkptr = Arc::new(Box::new(zk));
        let mut in_lock = ZkDistrLock::new(zkptr.clone(), "/test/lock".to_string(), "test".to_string(), LockType::Lock);
        // let in_lock_2 = InnerLock::new(zkptr.clone(),"/test/lock".to_string(),"test".to_string(), LockType::Lock);
        let result = in_lock.acquire().expect("debug 2");
        assert_eq!(result, true);
        let result2 = in_lock.release().expect("debug 3");
        assert_eq!(result2, true);
    }

    #[test]
    fn test_two_internal_lock(){
        let zkserver= "127.0.0.1:2181";
        let zk = ZooKeeper::connect(&zkserver, Duration::from_secs(15), DummyWatcher).expect("debug main 1");
        let zkptr = Arc::new(Box::new(zk));
        let mut in_lock_1 = ZkDistrLock::new(zkptr.clone(), "/test/lock2".to_string(), "main".to_string(), LockType::Lock);

        let zkptr_thread = zkptr.clone();
        let handle = thread::spawn(|| -> Instant{
            thread::sleep_ms(1000);
            let mut in_lock_2 = ZkDistrLock::new(zkptr_thread, "/test/lock2".to_string(), "thread".to_string(), LockType::Lock);

            let result = in_lock_2.acquire().expect("debug thread 2");
            assert_eq!(result, true);


            let got_lock = Instant::now();
            thread::sleep_ms(1000);

            let result2 = in_lock_2.release().expect("debug thread 3");
            assert_eq!(result2, true);

            got_lock
        });

        let result = in_lock_1.acquire().expect("debug main 2");
        assert_eq!(result, true);

        let got_lock = Instant::now();
        thread::sleep_ms(5000);
        let elapse = got_lock.elapsed();

        let result2 = in_lock_1.release().expect("debug main 3");
        assert_eq!(result2, true);

        let thread_got_lock = handle.join().expect("debug main 4");

        let time_diff = thread_got_lock - got_lock;
        assert!(time_diff > elapse);
    }

    #[test]
    fn test_two_lock(){
        let zkserver= "127.0.0.1:2181";
        let zk = ZooKeeper::connect(&zkserver, Duration::from_secs(15), DummyWatcher).expect("debug main 1");
        let zkptr = Arc::new(Box::new(zk));
        let mut in_lock_1 = Lock::new(zkptr.clone(),"/test/lock3".to_string(),"main".to_string(), LockType::Lock);

        let zkptr_thread = zkptr.clone();
        let handle = thread::spawn(|| -> Instant{
            thread::sleep_ms(1000);
            let mut in_lock_2 = Lock::new(zkptr_thread,"/test/lock3".to_string(),"thread".to_string(), LockType::Lock);

            let result = in_lock_2.acquire().expect("debug thread 2");
            assert_eq!(result, true);


            let got_lock = Instant::now();
            thread::sleep_ms(1000);

            let result2 = in_lock_2.release().expect("debug thread 3");
            assert_eq!(result2, true);

            got_lock
        });

        let result = in_lock_1.acquire().expect("debug main 2");
        assert_eq!(result, true);

        let got_lock = Instant::now();
        thread::sleep_ms(5000);
        let elapse = got_lock.elapsed();

        let result2 = in_lock_1.release().expect("debug main 3");
        assert_eq!(result2, true);

        let thread_got_lock = handle.join().expect("debug main 4");

        let time_diff = thread_got_lock - got_lock;
        assert!(time_diff > elapse);
    }

    #[test]
    fn test_critical_section() {
        let zkserver= "127.0.0.1:2181";
        let zk = ZooKeeper::connect(&zkserver, Duration::from_secs(15), DummyWatcher).expect("debug main 1");
        let zkptr = Arc::new(Box::new(zk));
        let cs = ZkCriticalSection::new(zkptr.clone(), "/test/lock4".to_string(), "".to_string());
        let mut thread_handle = vec!();

        /// unsafe in thread
        // let mut counter = Arc::new(Cell::new(0));

        let mut counter = Arc::new(atomic::AtomicUsize::new(0));



        for i in 0..10{
            let mut cs_tmp = cs.clone();
            let mut counter_tmp = counter.clone();
            let handle = thread::spawn( move ||{
                let func = || { (*counter_tmp).store( (*counter_tmp).load(Ordering::Relaxed) + 1, Ordering::Relaxed) };
                cs_tmp.lock()
                      .unwrap_or_else(|e| e.into_inner())
                      .guard(func);
            });
            thread_handle.push(handle);
        }

        for h in thread_handle{
            h.join().expect("debug main 2");
        }

        //assert_eq!((*counter).get(), 10);
        assert_eq!(counter.load(Ordering::Relaxed), 10);
    }

    // try to make cancel work, but really lack knowledge of how sharing struct between thread
    // this is being counterproductive, I should just leave it here, try learn more.

//    #[test]
//    fn test_cancel() {
//
//        //let handle = thread::scoped(move || {
//        crossbeam::scope(|scope| {
//            let zkserver= "127.0.0.1:2181";
//            let zk = ZooKeeper::connect(&zkserver, Duration::from_secs(15), DummyWatcher).expect("debug main 1");
//            let zkptr = Arc::new(Box::new(zk));
//
//            let mut lock_1 = ZkDistrLock::new(zkptr.clone(), "/test/lock5".to_string(), "main_cancel".to_string(), LockType::Lock);
//            let cancel = (&lock_1).cancel();
//
//            let lock1_handle = Arc::new(Box::new(lock_1));
//            let lock1_handle_2 = lock1_handle.clone();
//
//            let zkptr_thread = zkptr.clone();
//
//
//            scope.spawn(move |_| {
//                let mut in_lock_2 = ZkDistrLock::new(zkptr_thread, "/test/lock5".to_string(), "thread_cancel".to_string(), LockType::Lock);
//
//                let result = in_lock_2.acquire().expect("debug thread 2");
//                assert_eq!(result, true);
//                thread::sleep_ms(2000);
//
////                lock1_handle_2
////                    .cancel();
//                //.expect("debug 2");
//
//                cancel();
//
//                thread::sleep_ms(5000);
//
//                let result2 = in_lock_2.release().expect("debug thread 3");
//                assert_eq!(result2, true);
//            });
//
//            thread::sleep_ms(1000);
//
//            let result = lock1_handle
//                .acquire()
//                .expect("debug main 2.1");
//
//            assert_eq!(result, false);
//
//
//            let result2 = lock1_handle
//                .release()
//                .expect("debug main 3.1");
//            assert_eq!(result2, true);
//
//        });
//    }

}