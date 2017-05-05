//! Parsing of nsswitch.conf.
//!
//! Parsing herein follows the `nsswitch.conf` file used by glibc 2.

use std::{error, fmt, fs, io};
use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;


//------------ Conf ----------------------------------------------------------

/// The name service switch configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Conf {
    databases: HashMap<Database, Vec<Rule>>,
}


impl Conf {
    pub fn new() -> Self {
        Conf {
            databases: HashMap::new()
        }
    }

    pub fn database(&self, db: &Database) -> Option<&[Rule]> {
        self.databases.get(db).map(|v| v.as_ref())
    }
}


/// # Parsing Conf File
///
impl Conf {
    /// Parse a conf file.
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::parse(&mut fs::File::open(path)?)
    }

    /// Parse a conf from a reader.
    pub fn parse<R: io::Read>(reader: &mut R) -> Result<Self, Error> {
        use std::io::BufRead;

        let mut res = Conf::new();
        for line in io::BufReader::new(reader).lines() {
            let _ = res.parse_line(&mut line?);
        }
        Ok(res)
    }

    fn parse_line(&mut self, line: &mut str) -> Result<(), Error> {
        /// Quick workaround: Make everything lowercase.
        line.make_ascii_lowercase();

        /// Remove comments, strip white space, and return early on empty.
        let line: &str = match line.find('#') {
            Some(pos) => line.split_at(pos).0,
            None => &line
        };
        let line = line.trim();
        if line.is_empty() { return Ok(()) }
        let mut words = line.split_whitespace();

        /// First word is the database followed by a colon.
        let db = words.next().ok_or(Error::ParseError)?;
        if !db.ends_with(':') {
            return Err(Error::ParseError);
        }
        let db = db.trim_right_matches(':');
        let db = Database::from_str(db)?;

        /// All following words are rules.
        ///
        /// We can’t use collect() here because of the error handling. Or
        /// can we?
        let mut rules = Vec::new();
        for word in words {
            rules.push(Rule::from_str(word)?)
        }

        self.databases.insert(db, rules);
        Ok(())
    }
}


//------------ Rule ----------------------------------------------------------

/// A lookup rule for a single database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Rule {
    Service(Service),
    Action(Status, Action),
}


impl FromStr for Rule {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('[') {
            if !s.ends_with(']') {
                return Err(Error::ParseError)
            }
            let mut iter = s.trim_left_matches('[')
                            .trim_right_matches(']');
                            .splitn(2, '=');
            let status = iter.next().ok_or(Error::ParseError)?;
            let action = iter.next().ok_or(Error::ParseError)?;
            Ok(Rule::Action(Status::from_str(status)?,
                            Action::from_str(action)?))
        }
        else {
            Ok(Rule::Service(Service::from_str(s)?))
        }
    }
}


//------------ Database ------------------------------------------------------

/// A database referenced in the name service configuration.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Database {
    /// The hosts database.
    Hosts,

    /// The networks database.
    Networks,

    /// The protocols database.
    Protocols,

    /// The services database.
    Services,

    /// Some other database not supported by this crate.
    Other(String),
}


impl FromStr for Database {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "hosts" => Database::Hosts,
            "networks" => Database::Networks,
            "protocols" => Database::Protocols,
            "services" => Database::Services,
            _ => Database::Other(s.into()),
        })
    }
}

impl From<Database> for String {
    fn from(db: Database) -> Self {
        match db {
            Database::Hosts => "hosts".into(),
            Database::Networks => "networks".into(),
            Database::Protocols => "protocols".into(),
            Database::Services => "services".into(),
            Database::Other(db) => db,
        }
    }
}

impl AsRef<str> for Database {
    fn as_ref(&self) -> &str {
        match *self {
            Database::Hosts => "hosts",
            Database::Networks => "networks",
            Database::Protocols => "protocols",
            Database::Services => "services",
            Database::Other(ref db) => db,
        }
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}


//------------ Status --------------------------------------------------------

/// The status value of the name service configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// `"success"`.
    ///
    /// No error occurred and the requested entry is returned.
    Success,

    /// `"notfound"`
    ///
    /// The lookup succeeded but not entry was found.
    NotFound,

    /// `"unavail"`
    ///
    /// The service is permanently unavailable.
    Unavail,

    /// `"tryagain"`
    ///
    /// The service is temporarily unavailable.
    TryAgain
}


impl FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(Status::Success),
            "notfound" => Ok(Status::NotFound),
            "unavail" => Ok(Status::Unavail),
            "tryagain" => Ok(Status::TryAgain),
            _ => Err(Error::ParseError),
        }
    }
}

impl From<Status> for &'static str {
    fn from(stat: Status) -> Self {
        match stat {
            Status::Success => "success",
            Status::NotFound => "notfound",
            Status::Unavail => "unavail",
            Status::TryAgain => "tryagain",
        }
    }
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &str {
        (*self).into()
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}


//------------ Action --------------------------------------------------------

/// An action value in the name service configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// `"return"`
    ///
    /// Return a result now.
    Return,

    /// `"continue"`
    ///
    /// Continue with the next lookup.
    Continue,

    /// `"merge"`
    ///
    /// Merge the result from previous lookups with any successful
    /// consecutive lookup.
    Merge
}


impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "return" => Ok(Action::Return),
            "continue" => Ok(Action::Continue),
            "merge" => Ok(Action::Merge),
            _ => Err(Error::ParseError)
        }
    }
}

impl From<Action> for &'static str {
    fn from(action: Action) -> Self {
        match action {
            Action::Return => "return",
            Action::Continue => "continue",
            Action::Merge => "merge",
        }
    }
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &str {
        (*self).into()
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}


//------------ Service -------------------------------------------------------

/// A service value in the name service configuration.
///
/// This enum contains variants for all the service values we know how to
/// process internally. Additionally, the variant `Other` is used for all
/// unknown service values.
///
/// Note that not all service values are necessary valid for all databases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Service {
    /// `"compat"`
    ///
    /// Similar to `Files` but with additional information allowed in the
    /// files for some services.
    Compat,

    /// `"dns"`
    ///
    /// The DNS service. This is allowed for `Database::Hosts`.
    Dns,

    /// `"files"`
    ///
    /// The files service.
    Files,

    /// Any other service.
    Other(String),
}

impl FromStr for Service {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "compat" => Service::Compat,
            "dns" => Service::Dns,
            "files" => Service::Files,
            _ => Service::Other(s.into())
        })
    }
}

impl From<Service> for String {
    fn from(service: Service) -> Self {
        match service {
            Service::Compat => "compat".into(),
            Service::Dns => "dns".into(),
            Service::Files => "files".into(),
            Service::Other(service) => service
        }
    }
}

impl AsRef<str> for Service {
    fn as_ref(&self) -> &str {
        match *self {
            Service::Compat => "compat",
            Service::Dns => "dns",
            Service::Files => "files",
            Service::Other(ref service) => service,
        }
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}


//------------ Error and Result ----------------------------------------------

/// An error happend during parsing a hosts file.
#[derive(Debug)]
pub enum Error {
    /// The host file is kaputt.
    ParseError,

    /// Reading failed.
    IoError(io::Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError => "error parsing configuration",
            Error::IoError(ref e) => e.description(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
    }
}

/*
impl From<name::FromStrError> for Error {
    fn from(_: name::FromStrError) -> Error {
        Error::ParseError
    }
}

impl From<::std::num::ParseIntError> for Error {
    fn from(_: ::std::num::ParseIntError) -> Error {
        Error::ParseError
    }
}

impl From<net::AddrParseError> for Error {
    fn from(_: net::AddrParseError) -> Error {
        Error::ParseError
    }
}
*/

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;

        self.description().fmt(f)
    }
}


//============ Testing =======================================================

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        use std::io::Cursor;

        let mut conf = Cursor::new(
            "# /etc/nsswitch.conf\n\
             #\n\
             \n\
             passwd:         compat\n\
             gshadow:        files\n\
             \n\
             hosts:          files mdns4_minimal [NOTFOUND=return] dns myho\n\
             networks:       files\n\
             \n\
             protocols:      db files\n\
             services:       db files\n\
             rpc:            db files\n\
             \n\
             netgroup:       nis\n\
             ");
        let conf = Conf::parse(&mut conf).unwrap();
        assert_eq!(conf.database(&Database::Other("passwd".into())),
                   Some(&[Rule::Service(Service::Compat)][..]));
        assert_eq!(conf.database(&Database::Other("gshadow".into())),
                   Some(&[Rule::Service(Service::Files)][..]));
        assert_eq!(conf.database(&Database::Hosts),
                   Some(&[
                        Rule::Service(Service::Files),
                        Rule::Service(Service::Other("mdns4_minimal".into())),
                        Rule::Action(Status::NotFound, Action::Return),
                        Rule::Service(Service::Dns),
                        Rule::Service(Service::Other("myho".into()))
                   ][..]));
    }
}

