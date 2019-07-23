// #![deny(unused_mut)]
extern crate zookeeper;

extern crate uuid;

// use std::String;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::{Sender,Receiver};
// use std::Vec;
use self::uuid::Uuid;
use self::zookeeper::{Acl, CreateMode, WatchedEvent, Watcher, ZooKeeper,ZkResult};
use self::zookeeper::ZooKeeperExt;

// mod errors;
use errors::RcpError;

struct Lock{
    lock: Arc<Mutex<InnerLock>>,

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

struct InnerLock{
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
}

impl InnerLock{
    pub fn new(zk: Arc<Box<ZooKeeper>>, path : String, identifier : String, lock_type :LockType ) -> InnerLock{
        let (node_name, exclude_names)= match lock_type {
            LockType::Lock => ("__lock__", vec!["__lock__".to_string(),"__rlock__".to_string()] ),
            LockType::ReadLock => ("__rlock__", vec!["__lock__".to_string()] ),
            LockType::WriteLock => ("__lock__", vec!["__lock__".to_string(),"__rlock__".to_string()] ),
        };

        let uid = Uuid::new_v4();
        let prefix = uid.to_string()+node_name;

        let create_path = format!("{}/{}",&path, &prefix );

        let (tx, rx) = mpsc::channel();

        let mut lock = InnerLock{
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

        let node_path = self.zk.create(&self.create_path, vec![],
                            Acl::open_unsafe().clone(),
                            CreateMode::EphemeralSequential)?;

        self.node = (&node_path)[self.path.len() + 1..].to_string();

        println!("node {}",&self.node);


        loop{
            let is_first = self.is_first()?;
            println!("is first {}", is_first);
            if is_first {
                self.is_acquired = true;
                return  Ok(true)
            }

            let pre = self.predecessor()?;

            // self.zk.get_data_w(&pre, watch_change );
            // self.zk.get_data_w(&pre, |event| self.watch_change(event) );
            // self.zk.get_data_w(&pre, |event| self.watch_tx.send(()).unwrap() );
            self.wait_for_change();
        }
    }

    fn watch_change(&self, event : WatchedEvent) {
        self.watch_tx.send(()).unwrap();
    }

    fn wait_for_change(&self){
        self.change_rx.recv().unwrap();
    }

    fn get_sorted_children(&self)->Result<Vec<String>, RcpError>{
        let mut children = self.zk.get_children(&self.path, false)?;
        children.sort_by_key( |x|{
            for name in vec!["__lock__","__rlock__"]{
                match x.find(&name){
                    Some(pos) => &x[pos+name.len()..],
                    None => "~",
                };
            };
        });
        Ok(children)
    }

    fn is_first(&self)->Result<bool, RcpError>{
        let children = self.get_sorted_children()?;
        println!("{:?}",&children);

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
            None => {println!("bug happen");return Err(RcpError::InternalError)},
        };
        println!("debug pos {}",pos);

        Ok(children[pos -1].clone())
    }

    fn release(&mut self)->Result<bool, RcpError>{
        if ! &self.is_acquired {
            return Ok(false);
        }

        self.zk.delete(&format!("{}/{}",self.path,self.node), None)?;

        self.is_acquired = false;
        self.node = "".to_string();

        Ok(true)
    }
}

#[cfg(test)]

mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::time::Duration;

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

    #[test]
    fn test_internal_lock() {

        let zkserver= "127.0.0.1:2181";
        let zk = ZooKeeper::connect(&zkserver, Duration::from_secs(15), DummyWatcher).expect("debug 1");
        let zkptr = Arc::new(Box::new(zk));
        let mut in_lock = InnerLock::new(zkptr.clone(),"/test/lock".to_string(),"test".to_string(), LockType::Lock);
        // let in_lock_2 = InnerLock::new(zkptr.clone(),"/test/lock".to_string(),"test".to_string(), LockType::Lock);
        let result = in_lock.acquire().expect("debug 2");
        assert_eq!(result, true);
        let result2 = in_lock.release().expect("debug 3");
        assert_eq!(result2, true);
    }
}