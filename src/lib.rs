#[test]
fn it_works() {
    println!("It Works");
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}