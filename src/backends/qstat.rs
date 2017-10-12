extern crate core;
extern crate futures;
extern crate futures_spawn;
extern crate std;
extern crate serde_json;

use errors;
use models;

use serde_json::Value;
use std::collections::*;
use std::process::*;
use std::ops::Deref;
use self::futures::*;
use self::futures_spawn::*;

use models::*;

#[derive(Clone, Default, Serialize, Deserialize)]
struct QStatPlayer {
    name: String,
    score: Option<i64>,
    ping: Option<i64>,
    time: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct QStatServer {
    protocol: Option<String>,
    address: std::net::SocketAddr,
    status: Option<String>,
    hostname: Option<String>,
    name: Option<String>,
    gametype: Option<String>,
    map: Option<String>,
    numplayers: Option<i64>,
    maxplayers: Option<i64>,
    numspectators: Option<i64>,
    maxspectators: Option<i64>,
    ping: Option<i64>,
    rules: HashMap<String, String>,
    players: Vec<QStatPlayer>,
}

impl QStatServer {
    fn try_into(&self) -> errors::Result<ServerEntry> {
        let v = models::ServerEntry::new(self.address);

        Ok(v)
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct QStatResponse(Vec<QStatServer>);

impl Deref for QStatResponse {
    type Target = Vec<QStatServer>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Backend;

fn make_rulestring(rules: &HashMap<String, String>) -> String {
    rules.iter().fold(String::new(), |acc, kv| {
        acc + "," + kv.0 + "=" + kv.1
    })
}

fn make_qstat_cmd_params(
    master_type: &str,
    rules: &HashMap<String, String>,
    master_server_uri: &Vec<String>,
) -> Vec<String> {
    let rulestring = make_rulestring(rules);

    let mut cmd: Vec<String> = vec![];
    cmd.push("-json".to_string());
    cmd.push("-utf8".to_string());
    cmd.push("-maxsim".to_string());
    cmd.push("9999".to_string());
    cmd.push("-R".to_string());
    cmd.push("-P".to_string());
    cmd.push("-".to_string() + &master_type.to_lowercase() + &rulestring);
    cmd.push(master_server_uri.join(" "));

    cmd
}

fn get_string_array(v: &Value) -> errors::Result<Vec<String>> {
    let varray = match *v {
        Value::Array(ref e) => e,
        _ => {
            bail!(errors::ErrorKind::DataParseError(
                "Not a valid array of strings".into(),
            ))
        }
    };
    let mut out = Vec::<String>::new();
    for val in varray {
        match *val {
            Value::String(ref s) => {
                out.push(s.clone());
            }
            _ => {
                bail!(errors::ErrorKind::DataParseError("Not a string".into()));
            }
        }
    }

    Ok(out)
}

fn parse(raw: &str, server_type: &str) -> errors::Result<ServerData> {
    let data: QStatResponse = try!(serde_json::from_str(raw));

    Ok(data.iter().fold(
        ServerData::default(),
        |mut acc, ref entry| {
            entry.try_into().map(|v| { acc.insert(v); });
            acc
        },
    ))
}

struct QuerySettings {
    qstat_path: String,
    master_type: String,
    server_type: String,
    master_server_uri: Vec<String>,
    qstat_game_type: Option<String>,
}

impl QuerySettings {
    fn try_from(v: &ConfStorage) -> errors::Result<QuerySettings> {
        Ok(Self {
            qstat_path: v.get_or_err("qstat_path")?,
            master_type: v.get_or_err("qstat_master_type")?,
            server_type: v.get_or_err("qstat_server_type")?,
            master_server_uri: v.get_or_err("master_server_uri")?,
            qstat_game_type: v.get("qstat_game_type".into())
                .and_then(|j| match *j {
                    Value::String(ref j) => j.into(),
                    _ => None,
                })
                .cloned(),
        })
    }
}

fn query(settings: &QuerySettings) -> errors::Result<ServerData> {

    let mut rules = HashMap::<String, String>::new();

    settings.qstat_game_type.as_ref().map(|v| {
        rules.insert("qstat_game_type".into(), v.clone());
    });
    let mut child = Command::new(&settings.qstat_path)
        .arg(
            make_qstat_cmd_params(&settings.master_type, &rules, &settings.master_server_uri)
                .join(" "),
        )
        .stdout(Stdio::piped())
        .spawn()?;

    let out = child.wait_with_output()?;
    if !out.status.success() {
        let desc = match out.status.code() {
            Some(code) => format!("QStat exited with status: {}", code),
            None => "QStat killed by a signal".into(),
        };
        return Err(
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, desc.as_str()).into(),
        );
    }

    parse(
        &String::from_utf8(out.stdout.clone())?,
        &settings.server_type,
    )
}

impl Backend {
    fn get_qstat_output(&self) {}
}

impl models::DataSource for Backend {
    fn query(&self, settings: &ConfStorage) -> Box<errors::FResult<ServerData>> {
        let s = match QuerySettings::try_from(settings) {
            Ok(v) => v,
            Err(e) => {
                return Box::new(future::result(Err(e)));
            }
        };

        Box::from(futures_spawn::NewThread.spawn(
            futures::lazy(move || query(&s)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_qstat_cmd_params() {
        let mut rules = HashMap::<String, String>::new();
        rules.insert("rule1".into(), "value1".into());
        rules.insert("rule2".into(), "value2".into());

        let master_server_uri = vec!["serverA".into(), "serverB".into()];

        let func = &make_qstat_cmd_params;
        let expectation: Vec<String> = vec![
            String::from("-json"),
            String::from("-utf8"),
            String::from("-maxsim"),
            String::from("9999"),
            String::from("-R"),
            String::from("-P"),
            String::from("-q3s") + &make_rulestring(&rules),
            String::from("serverA serverB"),
        ];
        assert_eq!(func("q3s", &rules, &master_server_uri), expectation);
    }

    #[test]
    fn test_parse() {
        let fixture = r##"
[
    {
        "protocol": "a2s",
        "address": "100.110.120.130:33333",
        "status": "online",
        "hostname": "100.110.120.130:33333",
        "name": "MyPreciousServer",
        "gametype": "cstrike",
        "map": "de_dust2",
        "numplayers": 14,
        "maxplayers": 33,
        "numspectators": 0,
        "maxspectators": 0,
        "ping": 10,
        "retries": 0,
        "rules": {
            "protocol": "11",
            "gamedir": "cstrike",
            "gamename": "Counter-Strike: Source",
            "bots": "0",
            "dedicated": "1",
            "sv_os": "linux",
            "secure": "1",
            "version": "3398447",
            "game_port": "33333",
            "game_tags": "increased_maxplayers,!weapon,alltalk,bhop,bunnyhopping,de,mg,minigames,no-steam,shop,startmoney"
        },
        "players": [
            {
                "name": "PlayerA",
                "score": 0,
                "time": "9s"
            },
            {
                "name": "PlayerB",
                "score": 10,
                "time": "1m37s"
            }
        ]
    }
]
"##;
        let func = &parse;
        let mut expectation = ServerData::default();
        expectation.insert(ServerEntry::new("100.110.120.130:33333".parse().unwrap()));
        assert_eq!(func(fixture, "a2s").unwrap(), expectation);
    }
}
