//! The dns source for the hosts database.

use std::{io, mem};
use std::net::IpAddr;
use domain::bits::DNameSlice;
use domain::resolv::Resolver;
use domain::resolv::error::Error;
use domain::resolv::lookup::host::{LookupHost, lookup_host};
use domain::resolv::lookup::addr::{LookupAddr, lookup_addr};
use futures::{Async, Future, Poll};
use tokio_core::reactor;
use super::HostEnt;


//------------ HostByName ----------------------------------------------------

pub struct HostByName(Result<LookupHost, Option<io::Error>>);

impl HostByName {
    pub fn new<N: AsRef<DNameSlice>>(name: N, reactor: &reactor::Handle)
                                     -> Self {
        HostByName(Ok(lookup_host(Resolver::new(reactor), name)))
    }
}

impl Future for HostByName {
    type Item = Option<HostEnt>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0 {
            Ok(ref mut lookup) => {
                match lookup.poll() {
                    Ok(Async::Ready(found)) => {
                        Ok(Async::Ready(Some(HostEnt {
                            name: format!("{}", found.canonical_name()),
                            aliases: Vec::new(),
                            addrs: found.iter().collect(),
                        })))
                    }
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Err(Error::Question(err))
                        => panic!("Question error: {}", err),
                    Err(Error::Io(err)) => Err(err),
                    _ => Ok(Async::Ready(None)),
                }
            }
            Err(ref mut inner) => {
                match mem::replace(inner, None) {
                    Some(err) => Err(err),
                    None => panic!("polling a resolved HostByName"),
                }
            }
        }
    }
}


//------------ HostByAddr ----------------------------------------------------

pub struct HostByAddr {
    addr: IpAddr,
    result: Result<LookupAddr, Option<io::Error>>,
}

impl HostByAddr {
    pub fn new(addr: IpAddr, reactor: &reactor::Handle) -> Self {
        HostByAddr {
            addr: addr,
            result: Ok(lookup_addr(Resolver::new(reactor), addr))
        }
    }
}

impl Future for HostByAddr {
    type Item = Option<HostEnt>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.result {
            Ok(ref mut lookup) => {
                match lookup.poll() {
                    Ok(Async::Ready(found)) => {
                        let mut iter = found.iter();
                        let name = match iter.next() {
                            None => return Ok(Async::Ready(None)),
                            Some(name) => format!("{}", name)
                        };
                        Ok(Async::Ready(Some(HostEnt {
                            name: name,
                            aliases: iter.map(|n| format!("{}", n)).collect(),
                            addrs: vec![self.addr],
                        })))
                    }
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Err(Error::Question(err))
                        => panic!("Question error: {}", err),
                    Err(Error::Io(err)) => Err(err),
                    _ => Ok(Async::Ready(None)),
                }
            }
            Err(ref mut inner) => {
                match mem::replace(inner, None) {
                    Some(err) => Err(err),
                    None => panic!("polling a resolved HostByAddr")
                }
            }
        }
    }
}


