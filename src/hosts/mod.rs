use std::{io, mem};
use std::net::IpAddr;
use std::str::FromStr;
use domain::bits::DNameBuf;
use futures::{Async, Future, Poll};
use tokio_core::reactor;


//============ Low-level API =================================================

pub mod dns;
pub mod files;


//============ High-level API ================================================

pub fn get_host_by_name(name: &str) -> Result<Option<HostEnt>, io::Error> {
    let mut core = reactor::Core::new()?;
    let handle = core.handle();
    core.run(poll_host_by_name(name, &handle))
}

pub fn get_host_by_addr(addr: IpAddr) -> Result<Option<HostEnt>, io::Error> {
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
                         -> HostByName {
    HostByName::new(name, reactor)
}

pub fn poll_host_by_addr(addr: IpAddr, reactor: &reactor::Handle)
                         -> HostByAddr {
    unimplemented!()
}


//------------ HostEnt -------------------------------------------------------

pub struct HostEnt {
    pub name: String,
    pub aliases: Vec<String>,
    pub addrs: Vec<IpAddr>,
}


//------------ HostByName ----------------------------------------------------

pub struct HostByName(ByNameInner);

enum ByNameInner {
    Files(HostEnt),
    Dns(dns::HostByName),
    Error(io::Error),
    Done,
}

impl HostByName {
    pub fn new(name: &str, reactor: &reactor::Handle) -> Self {
        if let Ok(addr) = IpAddr::from_str(name) {
            return HostByName(ByNameInner::Files(HostEnt {
                name: name.into(),
                aliases: Vec::new(),
                addrs: vec!(addr),
            }))
        }
        let name = match DNameBuf::from_str(name) {
            Ok(name) => name,
            Err(e) => {
                return HostByName(ByNameInner::Error(
                    io::Error::new(io::ErrorKind::Other, e)
                ))
            }
        };
        HostByName(match files::get_host_by_name(&name) {
            Ok(Some(ent)) => ByNameInner::Files(ent),
            Ok(None) => ByNameInner::Dns(dns::HostByName::new(name, reactor)),
            Err(err) => ByNameInner::Error(err),
        })
    }
}


impl Future for HostByName {
    type Item = Option<HostEnt>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0 {
            ByNameInner::Dns(ref mut lookup) => return lookup.poll(),
            _ => { }
        }
        match mem::replace(&mut self.0, ByNameInner::Done) {
            ByNameInner::Files(res) => Ok(Async::Ready(Some(res))),
            ByNameInner::Error(err) => Err(err),
            ByNameInner::Done => panic!("polling a resolved HostByName"),
            _ => panic!()
        }
    }
}


//------------ HostByAddr ----------------------------------------------------

pub struct HostByAddr;

impl Future for HostByAddr {
    type Item = Option<HostEnt>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        unimplemented!()
    }
}

