// mod how_you_hold_data_for_operations;

///associate greetings module with this crate
mod greetings;
extern crate hello_world_lib;
///Optionally oad each member of greetings
/*use greetings::default_greeting;
use greetings::spanish;
use greetings::french;*/
///Alternatively, use * to load them all
//use greetings::*;
///Alternatively, load them in one line
use greetings::{ spanish, french, english};
fn main() {
println!("Hello, world!");
println!("{}", english::default_greeting());
println!("{}", spanish::default_greeting());
println!("{}", french::default_greeting());
println!("{}", hello_world_lib::greeting_from_lib());
}
