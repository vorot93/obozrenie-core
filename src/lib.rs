extern crate geoip;
extern crate rgs_models;
extern crate serde_json;

// So it goes like this:
// MyGame --- Backend1 - Setting A --- default (determines data type)
//         |                        |
//         |                        -- metadata (HashMap<String, String>)
//         |           - Setting B
//         |           - Setting C
//         |
//         -- Backend2 - Setting A
//                     - Setting B
//                     - Setting C

mod backends;
mod launch;

use serde_json::Value;
use std::cmp::*;
use std::collections::*;
use std::ops::*;
use rgs_models::*;
use backends::*;
use launch::*;

// CONFIG

pub type GameID = String;

pub struct ConfigEntry {
    pub default: Value,
    pub metadata: HashMap<String, Value>,
}

pub struct GameListEntry {
    pub name: String,
    pub backends: HashMap<Backend, HashMap<String, ConfigEntry>>,
    pub launch_patterns: HashMap<LaunchPattern, HashMap<String, ConfigEntry>>,
}

pub struct GameList(HashMap<GameID, GameListEntry>);

// GENERAL STUFF

#[derive(Clone, Copy)]
pub enum QueryStatus {
    Empty,
    Working,
    Ready,
    Error,
}

pub struct ServerEntry(Server);

impl Deref for ServerEntry {
    type Target = Server;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for ServerEntry {
    fn eq(&self, other: &Self) -> bool { self.host == other.host }
}

impl Eq for ServerEntry {}

impl PartialOrd for ServerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.host.partial_cmp(&other.host)
    }
}

impl Ord for ServerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.host.cmp(&other.host)
    }
}

pub type ServerData = BTreeSet<ServerEntry>;

pub type ConfStorage = HashMap<String, Value>;

pub type QueryFunc = fn(GameID, ConfStorage) -> ServerData;

pub enum ConfType {
    Launcher,
    Backend,
    System,
}

impl ConfType {
    pub fn from_string(s: &str) -> Result<ConfType, &'static str> {
        match s {
            "launcher" => Ok(ConfType::Launcher),
            "backend"  => Ok(ConfType::Backend),
            "system"   => Ok(ConfType::System),
            _ => Err("Invalid type string"),
        }
    }
    pub fn to_string(&self) -> &str {
        match self {
            &ConfType::Launcher => "launcher",
            &ConfType::Backend  => "backend",
            &ConfType::System   => "system",
        }
    }
}

pub type Settings = HashMap<String, ConfStorage>;

pub struct GameEntry {
    pub status: QueryStatus,
    pub query_func: QueryFunc,
    pub servers: ServerData,
    pub settings: Settings,
}

fn default_query_func(_: GameID, _: ConfStorage) -> ServerData {
    return ServerData::new();
}

impl Default for GameEntry {
    fn default() -> GameEntry {
        GameEntry {
            status: QueryStatus::Empty,
            query_func: default_query_func,
            servers: ServerData::new(),
            settings: Settings::new(),
        }
    }
}

pub struct GameTable {
    pub data: HashMap<GameID, GameEntry>,
}

impl GameTable {
    fn new() -> GameTable {
        GameTable { data: HashMap::new() }
    }
}

pub struct Core {
    game_table: GameTable,
    geoip: Option<geoip::GeoIp>,
}

// CORE

impl Core {
    pub fn new(geoip: Option<geoip::GeoIp>) -> Core {
        let core = Core {game_table: GameTable::new(), geoip: geoip};
        core
    }

    pub fn refresh_servers(&self, id: GameID) {
    }

    pub fn read_game_lists(&self, data: GameList) -> Result<String, String> {
        Ok("Success.".to_string())
    }
}
