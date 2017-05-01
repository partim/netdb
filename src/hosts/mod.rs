/// The host name and IP address database.
///
/// This database provides queries for host names and IP addresses associated
/// with network hosts. It allows lookups based on a given host name or a
/// given IP address.

use std::{io, mem};
use std::net::IpAddr;
use std::str::FromStr;
use domain::bits::DNameBuf;
use futures::{Async, Future, Poll};
use tokio_core::reactor;


//============ Low-level API =================================================
//
// Currently private.

mod dns;
mod files;


//============ High-level API ================================================

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
/// The function waits for all necessary IO to resolve. Upon success, it
/// returns a `HostEnt` value if a host for the given name was found or
/// `Ok(None)` otherwise.
///
/// # Limitations
///
/// For this initial version of the crate, the lookup is a `files` lookup
/// first and only if that does fail to yield a result, a DNS query for
/// both A and AAAA records. This initial version also does not yet fill
/// the aliases list of the returned `HostEnt`.
pub fn get_host_by_name(name: &str) -> Result<Option<HostEnt>, io::Error> {
    let mut core = reactor::Core::new()?;
    let handle = core.handle();
    core.run(poll_host_by_name(name, &handle))
}

/// Returns host information for a given IP address.
///
/// The IP address can either be an IPv4 or IPv6 address. The function waits
/// for all necessary IO to resolve. Upon success, it
/// returns a `HostEnt` value if a host for the given name was found or
/// `Ok(None)` otherwise.
///
/// # Limitations
///
/// For this initial version of the crate, the lookup is a `files` lookup
/// first and only if that does fail to yield a result, a DNS query for
/// PTR records. This initial version also does not yet fill
/// the aliases list of the returned `HostEnt`.
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
/// Tokio reactor given by `reactor`.
///
/// # Limitations
///
/// For this initial version of the crate, the lookup is a `files` lookup
/// first and only if that does fail to yield a result, a DNS query for
/// both A and AAAA records. This initial version also does not yet fill
/// the aliases list of the returned `HostEnt`.
pub fn poll_host_by_name(name: &str, reactor: &reactor::Handle)
                         -> HostByName {
    HostByName::new(name, reactor)
}

/// Returns host information for a given IP address.
///
/// The IP address can either be an IPv4 or IPv6 address. The function returns
/// a future performing all necessary IO via the Tokio reactor given by
/// `reactor`.
///
/// # Limitations
///
/// For this initial version of the crate, the lookup is a `files` lookup
/// first and only if that does fail to yield a result, a DNS query for
/// PTR records. This initial version also does not yet fill
/// the aliases list of the returned `HostEnt`.
pub fn poll_host_by_addr(addr: IpAddr, reactor: &reactor::Handle)
                         -> HostByAddr {
    HostByAddr::new(addr, reactor)
}


//------------ HostEnt -------------------------------------------------------

/// The result of a host lookup.
///
/// > **Note.** This implementation is highly temporary. While will probably
/// > keep the semantics, the actual types may change. 
pub struct HostEnt {
    name: String,
    aliases: Vec<String>,
    addrs: Vec<IpAddr>,
}

impl HostEnt {
    /// The canoncial name of the host.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The aliases of the host.
    ///
    /// > **Note.** Best to assume this is a slice of `str`.
    pub fn aliases(&self) -> &[String] {
        self.aliases.as_ref()
    }

    /// The addresses of the host.
    pub fn addrs(&self) -> &[IpAddr] {
        self.addrs.as_ref()
    }
}


//------------ HostByName ----------------------------------------------------

/// The future returned by `poll_host_by_name()`.
///
/// Resolves into a `HostEnt` value if the lookup is successful or `None` if
/// there is no such name.
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
        if let ByNameInner::Dns(ref mut lookup) = self.0 {
            return lookup.poll();
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

/// The future returned by `poll_host_by_addr()`.
///
/// Resolves into a `HostEnt` value if the lookup is successful or `None` if
/// there is no such address.
pub struct HostByAddr(ByAddrInner);

enum ByAddrInner {
    Files(HostEnt),
    Dns(dns::HostByAddr),
    Error(io::Error),
    Done
}

impl HostByAddr {
    pub fn new(addr: IpAddr, reactor: &reactor::Handle) -> Self {
        HostByAddr(match files::get_host_by_addr(addr) {
            Ok(Some(ent)) => ByAddrInner::Files(ent),
            Ok(None) => ByAddrInner::Dns(dns::HostByAddr::new(addr, reactor)),
            Err(err) => ByAddrInner::Error(err),
        })
    }
}

impl Future for HostByAddr {
    type Item = Option<HostEnt>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let ByAddrInner::Dns(ref mut lookup) = self.0 {
            return lookup.poll();
        }
        match mem::replace(&mut self.0, ByAddrInner::Done) {
            ByAddrInner::Files(res) => Ok(Async::Ready(Some(res))),
            ByAddrInner::Error(err) => Err(err),
            ByAddrInner::Done => panic!("polling a resolved HostByAddr"),
            _ => panic!()
        }
    }
}

