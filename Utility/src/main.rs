extern crate uuid;

use uuid::Uuid;

mod zoo_lock;
mod errors;

fn utility_test()
{
    let uid = Uuid::new_v4();
    println!("{}", &uid);
    println!("{}", (&uid).to_string() + "--test");

    let mut v = vec!["123","121","122","~"];
    v.sort();
    println!("{:?}",&v);
}

fn main(){
    println!("this is utility");
    utility_test()
}