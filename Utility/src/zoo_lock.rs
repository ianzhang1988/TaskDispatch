use zookeeper::{Acl, CreateMode, Watcher, WatchedEvent, ZooKeeper};
use zookeeper::ZkError;
use std::sync::{Arc, Mutex};
use std::String;
use std::Vec;

struct Lock{
    lock: Arc<Mutex<InnerLock>>,

}

struct InnerLock{
    zk: ZooKeeper,
    is_acquired : bool,
    path : String,
    identifier : String,
    node_name : String,
    exclude_names: Vec<String>,
}

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

impl InnerLock{
    fn new(zk: ZooKeeper, path : String, identifier : String, lock_type :LockType ) -> InnerLock{
        let (node_name, exclude_names)= match lock_type {
            LockType::Lock => ("__lock__", vec![Strint("__lock__"),Strint("__rlock__")] ),
            LockType::ReadLock => ("__rlock__", vec![Strint("__lock__")] ),
            LockType::WriteLock => ("__lock__", vec![Strint("__lock__"),Strint("__rlock__")] ),
        };

        let mut lock = InnerLock{
            zk: zk,
            is_acquired : False,
            path: path,
            identifier : identifier,
            node_name: node_name,
            exclude_names: exclude_names,
        };

        lock
    }

    fn acquire(&self){
        self.zk.ensure_path(self.path)
    }

    fn get_sorted_children(&self){
        self.zk.get_children()
    }

    fn predecessor(&self){

    }
}