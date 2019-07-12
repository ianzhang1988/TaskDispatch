extern crate uuid;

use uuid::Uuid;

mod zoo_lock;


fn utility_test()
{
    let uid = Uuid::new_v4();
    println!("{}", &uid);
    println!("{}", (&uid).to_string() + "--test");
}

fn main(){
    println!("this is utility");
    utility_test()
}