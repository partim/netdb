extern crate netdb;

use std::env;
use netdb::hosts::get_host_by_name;

fn main() {
    let mut args = env::args();
    let cmd = args.next().unwrap();
    let name = match args.next() {
        None => {
            println!("Usage: {} <hostname>", cmd);
            return;
        }
        Some(name) => name
    };

    match get_host_by_name(&name) {
        Ok(Some(ent)) => {
            println!("{}", ent.name());
            if !ent.aliases().is_empty() {
                println!("  Aliases:");
                for name in ent.aliases() {
                    println!("     {}", name);
                }
            }
            println!("  Addresses:");
            for addr in ent.addrs() {
                println!("     {}", addr);
            }
        }
        Ok(None) => println!("Not found."),
        Err(err) => println!("Error: {:?}", err),
    }
}
