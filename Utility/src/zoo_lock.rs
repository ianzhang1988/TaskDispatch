// #![deny(unused_mut)]
extern crate zookeeper;

extern crate uuid;

// use std::String;
use std::sync::{Arc, Mutex};
// use std::Vec;
use self::uuid::Uuid;
use self::zookeeper::{Acl, CreateMode, WatchedEvent, Watcher, ZooKeeper,ZkResult};
use self::zookeeper::ZooKeeperExt;

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
    zk: ZooKeeper,
    is_acquired : bool,
    path : String,
    identifier : String,
    node_name : String,
    exclude_names: Vec<String>,
    ensure_path : bool,
    prefix:String,
    create_path:String,
    node:String,
}

impl InnerLock{
    fn new(zk: ZooKeeper, path : String, identifier : String, lock_type :LockType ) -> InnerLock{
        let (node_name, exclude_names)= match lock_type {
            LockType::Lock => ("__lock__", vec!["__lock__".to_string(),"__rlock__".to_string()] ),
            LockType::ReadLock => ("__rlock__", vec!["__lock__".to_string()] ),
            LockType::WriteLock => ("__lock__", vec!["__lock__".to_string(),"__rlock__".to_string()] ),
        };

        let uid = Uuid::new_v4();
        let prefix = uid.to_string()+node_name;

        let create_path = format!("{}/{}",&path, &prefix );

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
        };

        lock
    }

    fn acquire(&mut self)->ZkResult<()>{

        match self.zk.exists(&self.path, false){
            Ok(o)=> match o {
                None => self.zk.ensure_path(&self.path).unwrap(),
                _ => ()
            },
            Err(e) => return Err(e),
        };

        self.node = self.zk.create(&self.create_path, vec![],
                            Acl::open_unsafe().clone(),
                            CreateMode::EphemeralSequential)?;





        Ok(())

    }


    fn get_sorted_children(&self){
        //self.zk.get_children()?
    }

    fn predecessor(&self){

    }
}