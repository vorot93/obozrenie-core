#![recursion_limit="1024"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate newtype_derive;
#[macro_use]
extern crate error_chain;

extern crate serde;
extern crate serde_json;
extern crate geoip;
extern crate rgs_models;

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
mod errors;
mod launch;
mod models;

use serde_json::Value;
use std::collections::*;
use std::collections::hash_map::Entry::*;
use std::sync::Mutex;

use backends::*;
use launch::*;
use models::*;

pub type GameID = String;

// CONFIG

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

impl Default for QueryStatus {
    fn default() -> QueryStatus {
        QueryStatus::Empty
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ConfType {
    Launcher,
    Backend,
    System,
}

impl ConfType {
    pub fn from_string(s: &str) -> Result<ConfType, &'static str> {
        match s {
            "launcher" => Ok(ConfType::Launcher),
            "backend" => Ok(ConfType::Backend),
            "system" => Ok(ConfType::System),
            _ => Err("Invalid type string"),
        }
    }
    pub fn to_string(&self) -> &str {
        match self {
            &ConfType::Launcher => "launcher",
            &ConfType::Backend => "backend",
            &ConfType::System => "system",
        }
    }
}

pub type Settings = HashMap<ConfType, ConfStorage>;

pub struct GameEntry {
    pub status: QueryStatus,
    pub data_source: Box<models::DataSource>,
    pub servers: ServerData,
    pub settings: Settings,
}

impl Default for GameEntry {
    fn default() -> Self {
        Self {
            status: Default::default(),
            data_source: Box::new(MockDataSource),
            servers: Default::default(),
            settings: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct GameTable {
    data: Mutex<HashMap<GameID, GameEntry>>,
}

fn get_game_entry<'a>(
    obj: &'a mut HashMap<GameID, GameEntry>,
    id: &GameID,
) -> errors::Result<&'a mut GameEntry> {
    let id = id.clone();
    obj.get_mut(&id).ok_or(
        errors::ErrorKind::NoSuchGameError(id)
            .into(),
    )
}

impl GameTable {
    fn new() -> GameTable {
        GameTable::default()
    }

    fn exec<T, R>(&self, mut func: T) -> R
    where
        T: FnMut(&HashMap<GameID, GameEntry>) -> R,
    {
        let obj = self.data.lock().unwrap();
        func(&*obj)
    }

    fn exec_mut<T, R>(&mut self, mut func: T) -> R
    where
        T: FnMut(&mut HashMap<GameID, GameEntry>) -> R,
    {
        let mut obj = self.data.lock().unwrap();
        func(&mut *obj)
    }

    pub fn list_games(&self) -> BTreeSet<GameID> {
        self.exec(|data| {
            data.iter().fold(
                BTreeSet::<GameID>::new(),
                |mut acc, entry| {
                    acc.insert(entry.0.clone());
                    acc
                },
            )
        })
    }

    pub fn create_game_entry(&mut self, id: &GameID) -> errors::Result<()> {
        self.exec_mut(|mut data| {
            let id = id.clone();
            match data.entry(id.clone()) {
                Vacant(entry) => entry.insert(GameEntry::default()),
                Occupied(_) => {
                    bail!(errors::ErrorKind::GameExistsError(id));
                }
            };
            Ok(())
        })
    }

    pub fn remove_game_entry(&mut self, id: &GameID) -> errors::Result<()> {
        self.exec_mut(|mut data| {
            let id = id.clone();
            match data.entry(id.clone()) {
                Vacant(_) => {
                    bail!(errors::ErrorKind::NoSuchGameError(id));
                }
                Occupied(entry) => {
                    entry.remove_entry();
                }
            };
            Ok(())
        })
    }
    fn get_settings(&mut self, id: &GameID, t: ConfType) -> Result<ConfStorage, errors::Error> {
        self.exec_mut(|mut data| {
            let id = id.clone();
            let game_entry = try!(get_game_entry(&mut data, &id));

            match game_entry.settings.entry(t) {
                Vacant(e) => Ok(e.insert(ConfStorage::default()).clone()),
                Occupied(e) => Ok(e.get().clone()),
            }
        })
    }
}
pub struct Core {
    game_table: GameTable,
    geoip: Option<geoip::GeoIp>,
}

// CORE

impl Core {
    pub fn new(geoip: Option<geoip::GeoIp>) -> Core {
        let core = Core {
            game_table: GameTable::new(),
            geoip: geoip,
        };
        core
    }

    pub fn refresh_servers(&mut self, id: GameID) {
        let settings = self.game_table.get_settings(&id, ConfType::Backend);
    }

    pub fn read_game_lists(&mut self, data: GameList) -> Result<String, String> {
        Ok("Success.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_game_table() {
        let obj = GameTable::default();
    }
}
