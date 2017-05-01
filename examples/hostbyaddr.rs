extern crate netdb;

use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use netdb::hosts::get_host_by_addr;

fn main() {
    let mut args = env::args();
    let cmd = args.next().unwrap();
    let addr = match args.next() {
        None => {
            println!("Usage: {} <addr>", cmd);
            return;
        }
        Some(addr) => addr
    };

    let addr = match IpAddr::from_str(&addr) {
        Ok(addr) => addr,
        Err(err) => {
            println!("Not an address: {:?}", err);
            return;
        }
    };

    match get_host_by_addr(addr) {
        Ok(Some(ent)) => {
            println!("{}", addr);
            println!("  {}", ent.name());
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


