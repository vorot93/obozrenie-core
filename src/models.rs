extern crate std;
extern crate futures;
extern crate rgs_models;
extern crate serde_json;

use errors;

use rgs_models::*;
use serde_json::Value;
use std::cmp::*;
use std::collections::*;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct ServerEntry(Server);

impl Deref for ServerEntry {
    type Target = Server;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for ServerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.addr.eq(&other.addr)
    }
}

impl Eq for ServerEntry {}

impl std::hash::Hash for ServerEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

impl ServerEntry {
    pub fn new(addr: std::net::SocketAddr) -> Self {
        Self { 0: rgs_models::Server::new(addr) }
    }
}

pub type ServerData = HashSet<ServerEntry>;

pub trait Config<T> {
    fn get_or_err(&self, k: &str) -> errors::Result<T>;
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
    fn get_or_err(&self, k: &str) -> errors::Result<Value> {
        match self.get(k) {
            Some(v) => Ok(v.clone()),
            None => Err(errors::ErrorKind::InvalidSettingKeyError(k.into()).into()),
        }
    }
}
impl Config<String> for ConfStorage {
    fn get_or_err(&self, k: &str) -> errors::Result<String> {
        match self.get_or_err(k)? {
            Value::String(s) => Ok(s),
            _ => Err(errors::ErrorKind::SettingTypeMismatchError(k.into()).into()),
        }
    }
}
impl Config<Vec<String>> for ConfStorage {
    fn get_or_err(&self, k: &str) -> errors::Result<Vec<String>> {
        let arr = match self.get_or_err(k)? {
            Value::Array(v) => v,
            _ => {
                return Err(errors::ErrorKind::SettingTypeMismatchError(k.into()).into());
            }
        };
        Ok(arr.iter()
            .map(|ref x| match x.as_str() {
                Some(v) => Ok(String::from(v)),
                None => {
                    Err(
                        errors::ErrorKind::SettingTypeMismatchError(
                            format!("Multi-type array detected: {}", k),
                        ).into(),
                    )
                }
            })
            .collect::<errors::Result<Vec<String>>>()?)
    }
}

impl Config<bool> for ConfStorage {
    fn get_or_err(&self, k: &str) -> errors::Result<bool> {
        match self.get_or_err(k)? {
            Value::Bool(v) => Ok(v),
            _ => Err(errors::ErrorKind::SettingTypeMismatchError(k.into()).into()),
        }
    }
}

pub trait DataSource {
    fn query(&self, &ConfStorage) -> Box<errors::FResult<ServerData>>;
}

pub struct MockDataSource;

impl DataSource for MockDataSource {
    fn query(&self, s: &ConfStorage) -> Box<errors::FResult<ServerData>> {
        Box::from(futures::future::ok(Default::default()))
    }
}
