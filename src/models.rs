extern crate std;
extern crate rgs_models;
extern crate serde_json;

use rgs_models::*;
use serde_json::Value;
use std::cmp::*;
use std::collections::*;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum Error {
    NullException(String),
    NoSuchGameError(String),
    GameExistsError(String),
    NotFoundError(String),
    IOError(String),
    InvalidUTF8Error(String),
    DataParseError(String),
    InvalidConfStorageError(String),
    InvalidSettingKeyError(String),
    SettingTypeMismatchError(String),
    BackendError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(format!("{}", e))
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::InvalidUTF8Error(format!("{}", e))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::DataParseError(format!("{}", e))
    }
}

custom_derive!{
    #[derive(NewtypeDeref, NewtypeDerefMut, Clone, Debug)]
    pub struct ServerEntry(Server);
}

impl PartialEq for ServerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
    }
}

impl Eq for ServerEntry {}

impl PartialOrd for ServerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.host.partial_cmp(&other.host)
    }
}

impl Ord for ServerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.addr.cmp(&other.addr)
    }
}

pub type ServerData = BTreeSet<ServerEntry>;

pub trait Config<T> {
    fn get_or_err(&self, k: &str) -> Result<T, Error>;
}

#[derive(Clone, Default)]
pub struct ConfStorage(HashMap<String, Value>);
impl Deref for ConfStorage {
    type Target = HashMap<String, Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Config<Value> for ConfStorage {
    fn get_or_err(&self, k: &str) -> Result<Value, Error> {
        let k_str = String::from(k);
        Ok(try!(self.get(k).ok_or(Error::InvalidSettingKeyError(k_str))).clone())
    }
}
impl Config<String> for ConfStorage {
    fn get_or_err(&self, k: &str) -> Result<String, Error> {
        let value: Value = try!(self.get_or_err(k));
        Ok(String::from(try!(value.as_str().ok_or(Error::SettingTypeMismatchError(k.into())))))
    }
}
impl Config<Vec<String>> for ConfStorage {
    fn get_or_err(&self, k: &str) -> Result<Vec<String>, Error> {
        let value: Value = try!(self.get_or_err(k));
        let arr = try!(value.as_array().ok_or(Error::SettingTypeMismatchError(k.into())));
        Ok(try!(arr.iter()
            .map(|ref x| match x.as_str() {
                Some(v) => Ok(String::from(v)),
                None => {
                    Err(Error::SettingTypeMismatchError(format!("Multi-type array detected: {}",
                                                                k)))
                }
            })
            .collect::<Result<Vec<String>, Error>>()))
    }
}

impl Config<bool> for ConfStorage {
    fn get_or_err(&self, k: &str) -> Result<bool, Error> {
        let value: Value = try!(self.get_or_err(k));
        Ok(try!(value.as_bool().ok_or(Error::SettingTypeMismatchError(k.into()))))
    }
}

pub struct QueryFunc(Box<Fn(ConfStorage) -> ServerData>);
impl Deref for QueryFunc {
    type Target = Box<Fn(ConfStorage) -> ServerData>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for QueryFunc {
    fn default() -> QueryFunc {
        QueryFunc(Box::new(|_| ServerData::default()))
    }
}
