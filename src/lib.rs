use std::collections::HashMap;
use std::collections::LinkedList;

#[derive(Clone, Copy)]
pub enum QueryStatus {
    Empty,
    Working,
    Ready,
    Error,
}

#[derive(Clone)]
pub struct Player {
    pub name: Option<String>,
    pub info: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Server {
    pub name: Option<String>,
    pub country: Option<String>,
    pub game_mod: Option<String>,
    pub game_type: Option<String>,
    pub need_pass: Option<bool>,
    pub secure: Option<bool>,
    pub player_count: Option<u64>,
    pub player_limit: Option<u64>,
    pub spectator_count: Option<u64>,
    pub spectator_limit: Option<u64>,
    pub terrain: Option<String>,
    pub ping: Option<u64>,
    pub rules: HashMap<String, String>,
    pub players: LinkedList<Player>,
}

type ServerData = std::collections::HashMap<String, Server>;
type GameID = String;

type ConfStorage = std::collections::HashMap<String, String>;

type QueryFunc = fn(GameID, ConfStorage) -> ServerData;

pub struct GameEntry {
    pub status: QueryStatus,
    pub query_func: QueryFunc,
    pub servers: ServerData,
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
        }
    }
}

pub struct GameTable {
    pub data: HashMap<GameID, GameEntry>,
}

pub struct Core {
}
