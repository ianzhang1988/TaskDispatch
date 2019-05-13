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

fn main() {

    let start = SystemTime::now();

    let zk = ZooKeeper::connect("10.19.17.188:2181,10.18.29.181:2181,10.19.16.30:2181", Duration::from_secs(15), LoggingWatcher).unwrap();
    zk.add_listener(|zk_state| println!("New ZkState is {:?}", zk_state));
    let children = zk.get_children("/", true).unwrap();
    for i in &children{
        println!("item / -> {:?}", i);
    }

    println!("children of / -> {:?}", children);
    println!("children of / -> {:?}", children);

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
