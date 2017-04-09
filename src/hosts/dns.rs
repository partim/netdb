//! The dns source for the hosts database.

use std::{io, mem};
use std::net::IpAddr;
use domain::bits::DNameSlice;
use domain::resolv::Resolver;
use domain::resolv::error::Error;
use domain::resolv::lookup::host::{lookup_host, FoundHosts, LookupHost};
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
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Ok(Async::Ready(found))
                        => Ok(Async::Ready(Some(found.into()))),
                    Err(Error::Question(err))
                        => panic!("Question error: {}", err),
                    Err(Error::Io(err)) => Err(err),
                    _ => Ok(Async::Ready(None)),
                }
            }
            Err(ref mut inner) => {
                match mem::replace(inner, None) {
                    Some(err) => Err(err),
                    None => panic!("polling a resolved HostByname"),
                }
            }
        }
    }
}


//------------ HostByAddr ----------------------------------------------------

pub struct HostByAddr;

impl HostByAddr {
    pub fn new(addr: IpAddr, reactor: &reactor::Handle) -> Self {
        unimplemented!()
    }
}

impl Future for HostByAddr {
    type Item = Option<HostEnt>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unimplemented!()
    }
}


//------------ Extension for HostEnt -----------------------------------------

impl From<FoundHosts> for HostEnt {
    fn from(found: FoundHosts) -> HostEnt {
        HostEnt {
            name: format!("{}", found.canonical_name()),
            aliases: Vec::new(),
            addrs: found.iter().collect(),
        }
    }
}

