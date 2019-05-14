#![deny(unused_mut)]
extern crate zookeeper;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::str;
use std::thread;
use std::time::Duration;
use zookeeper::{Acl, CreateMode, Watcher, WatchedEvent, ZooKeeper};
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread::sleep;
use std::process::Command;

struct LoggingWatcher;
impl Watcher for LoggingWatcher {
    fn handle(&self, e: WatchedEvent) {
        info!("{:?}", e)
    }
}

struct DummyWatcher;
impl Watcher for DummyWatcher {
    fn handle(&self, e: WatchedEvent) {
        ()
    }
}

//fn deleteall( &zk : ZooKeeper, path : str ){
//}

fn create_znode(zkserver : String, basepath : String) -> (u128, i32) {
    let start = SystemTime::now();

    let zk = ZooKeeper::connect(&zkserver, Duration::from_secs(15), DummyWatcher).unwrap();

    let path = zk.create(&basepath,
                                                    vec![],
                                                    Acl::open_unsafe().clone(),
                                                    CreateMode::Persistent).expect("create failed");

    let loop_num = 10000;

    for i in 0..loop_num {
        let path = zk.create(&format!("{}/{:010}",basepath, i),
                                                    vec![],
                                                    Acl::open_unsafe().clone(),
                                                    CreateMode::Persistent);
    }

    let difference = start.elapsed().expect("Time went backwards");
    (difference.as_millis(), loop_num)
}

fn test_create( zkserver : String, basepath : String, client_num : i32 ) {
    let start = SystemTime::now();

    let mut vec = Vec::new();

    for i in 1..client_num {
        let basepath_tmp = basepath.clone();
        let zkserver_tmp = zkserver.clone();
        let child = thread::spawn(  move|| {
            create_znode(zkserver_tmp,format!("{}/{:010}",basepath_tmp,i) )
        });
        vec.push(child);
    }

    let mut time_sum : u128=0;
    let mut num_sum = 0;
    let mut idx = 0;
    for i in vec {
        let (time, num) = i.join().unwrap();
        time_sum+=time;
        num_sum+=num;
        println!("avrage call time {}:{}",idx,time as f64/num as f64);
        idx+=1;
    }

    println!("----------------------\navrage call time {}",time_sum as f64/num_sum as f64);

    let difference = start.elapsed().expect("Time went backwards");
    println!("test_create run time {:?}",difference)
}

fn test_thread( input : String, other : String ) -> String{
    format!("{} {}", input , other)
}

fn test_thread_func(content:String){
    let child = thread::spawn(  || {
            test_thread(content,String::from("hi"))
        });

    let res = child.join().expect("thread join error");
    println!("{}",res)
}

fn main() {

    // test_thread_func(String::from("zhang"));
//    let (time, num) = create_znode("10.19.17.188:2181,10.18.29.181:2181,10.19.16.30:2181", "/test/10");
//    println!("time {} num {} avg {}", time, num , time as f64 / num as f64);

    test_create("10.19.17.188:2181,10.18.29.181:2181,10.19.16.30:2181".to_string(), "/test".to_string(), 100);

    return;

    let start = SystemTime::now();

    let zk = ZooKeeper::connect("10.19.17.188:2181,10.18.29.181:2181,10.19.16.30:2181", Duration::from_secs(15), LoggingWatcher).unwrap();
    zk.add_listener(|zk_state| println!("New ZkState is {:?}", zk_state));
    let children = zk.get_children("/", true).unwrap();
    for i in &children{
        println!("item / -> {:?}", i);
    }

    println!("children of / -> {:?}", children);
    println!("children of / -> {:?}", children);

    let children2 = zk.get_children("/nopath", true);
    match children2{
        Ok(lst)=>println!("Ok {:?}",lst),
        Err(e)=>println!("error {:?}",e)
    }

    let children3 = zk.get_children("/test", true).expect("get children error");
    println!("children3 {:?}", children3);

//    let children4 = zk.get_children("/nopath", true).expect("get children error");
//    println!("children3 {:?}", children4);

    // println!("nopath {:?}", children2);

    let mut vec = Vec::new();

    for i in 1..100 {
        let child = thread::spawn( move || {
            println!("number {}",i);
            i
        });
        vec.push(child);
    }

    println!("---------------");

    for i in vec {
        let res = i.join().unwrap();
        println!( "result  {}", res );
    }

    // sleep(Duration::from_secs(1));

//    let end = SystemTime::now();
//    let difference = end.duration_since(start)
//        .expect("Time went backwards");

    let mut difference = start.elapsed()
        .expect("Time went backwards");
    println!("{:?}", difference);

    let output = if cfg!(target_os = "windows") {
    Command::new("cmd")
            .args(&["/C", "ffmpeg -i D:\\Temp\\rouge1_tv_P1080_265_crf.mp4"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
                .arg("-c")
                .arg("echo hello")
                .output()
                .expect("failed to execute process")
    };

    let hello = output.stderr;
    println!("{}", str::from_utf8(&hello).expect("parse string error"));


    difference = start.elapsed().expect("Time went backwards");
    println!("{:?}", difference);
}
