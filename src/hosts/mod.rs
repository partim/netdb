use std::{io, mem};
use std::net::IpAddr;
use std::str::FromStr;
use domain::bits::DNameBuf;
use domain::resolv::Resolver;
use domain::resolv::lookup::host::{lookup_host, FoundHosts, LookupHost};
use futures::{Async, Future, Poll};
use tokio_core::reactor;


//============ Low-level API =================================================

pub mod files;


//============ High-level API ================================================

pub fn get_host_by_name(name: &str) -> Result<HostEnt, io::Error> {
    let mut core = reactor::Core::new()?;
    let handle = core.handle();
    core.run(poll_host_by_name(name, &handle))
}

pub fn get_host_by_addr(addr: IpAddr) -> Result<HostEnt, io::Error> {
    let mut core = reactor::Core::new()?;
    let handle = core.handle();
    core.run(poll_host_by_addr(addr, &handle))
}

/// Returns host information for a given host name.
///
/// The name is either a hostname, an IPv4 or IPv6 address in its standard
/// text notation. In the latter two cases, no lookups are performed and a
/// `HostEnt` is returned with `name` as the canonical name, the parsed
/// address as the sole address, and no aliases.
///
/// Otherwise the name is interpreted as a host name and lookups according to
/// the system configuraition are performed.
///
/// The function returns a future that performes all necessary IO via the
/// tokio reactor given by `reactor`.
///
/// # Limitations
///
/// For this initial version of the crate, the lookup is a `files` lookup
/// first and only if that does not succeed, a DNS query for both A and AAAA
/// records. This initial version also does not yet fill the aliases list of
/// the returned `HostEnt`.
pub fn poll_host_by_name(name: &str, reactor: &reactor::Handle)
                         -> HostByNameFuture {
    HostByNameFuture::new(name, reactor)
}

pub fn poll_host_by_addr(addr: IpAddr, reactor: &reactor::Handle)
                         -> HostByAddrFuture {
    unimplemented!()
}


//------------ HostEnt -------------------------------------------------------

pub struct HostEnt {
    pub name: String,
    pub aliases: Vec<String>,
    pub addrs: Vec<IpAddr>,
}

impl From<FoundHosts> for HostEnt {
    fn from(found: FoundHosts) -> HostEnt {
        HostEnt {
            name: format!("{}", found.canonical_name()),
            aliases: Vec::new(),
            addrs: found.iter().collect(),
        }
    }
}

//------------ HostByNameFuture ----------------------------------------------

pub struct HostByNameFuture(ByNameInner);

enum ByNameInner {
    Dns(LookupHost),
    Error(io::Error),
    Done,
}

impl HostByNameFuture {
    pub fn new(name: &str, reactor: &reactor::Handle) -> Self {
        let name = match DNameBuf::from_str(name) {
            Ok(name) => name,
            Err(e) => {
                return HostByNameFuture(ByNameInner::Error(
                    io::Error::new(io::ErrorKind::Other, e)
                ))
            }
        };
        HostByNameFuture(ByNameInner::Dns(lookup_host(Resolver::new(reactor),
                                                      name)))
    }
}


impl Future for HostByNameFuture {
    type Item = HostEnt;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0 {
            ByNameInner::Dns(ref mut lookup) => {
                let found = match lookup.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(found)) => found,
                    Err(err) => {
                        return Err(io::Error::new(io::ErrorKind::Other, err))
                    }
                };
                return Ok(Async::Ready(found.into()))
            }
            _ => { }
        }
        match mem::replace(&mut self.0, ByNameInner::Done) {
            ByNameInner::Error(err) => Err(err),
            ByNameInner::Done => panic!("polling a resolved HostByNameFuture"),
            _ => panic!()
        }
    }
}


//------------ HostByAddrFuture ----------------------------------------------

pub struct HostByAddrFuture;

impl Future for HostByAddrFuture {
    type Item = HostEnt;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unimplemented!()
    }
}

