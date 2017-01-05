extern crate core;
extern crate std;
extern crate serde_json;

use serde_json::Value;
use std::collections::*;
use std::process::*;
use std::ops::Deref;

use models::*;

#[derive(Clone, Default, Serialize, Deserialize)]
struct QStatPlayer {
    name: String,
    score: Option<String>,
    ping: Option<i64>,
    time: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct QStatServer {
    protocol: Option<String>,
    address: Option<String>,
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

#[derive(Clone, Default, Serialize, Deserialize)]
struct QStatResponse(Vec<QStatServer>);

impl Deref for QStatResponse {
    type Target = Vec<QStatServer>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn make_rulestring(rules: &HashMap<String, String>) -> String {
    rules.iter().fold(String::new(), |acc, kv| acc + "," + kv.0 + "=" + kv.1)
}

fn make_qstat_cmd_params(master_type: &str,
                         rules: &HashMap<String, String>,
                         master_server_uri: &Vec<String>)
                         -> Vec<String> {
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

fn get_string_array(v: &Value) -> Result<Vec<String>, Error> {
    let varray = try!(v.as_array()
        .ok_or(Error::DataParseError("Not a valid array of strings".into())));
    let mut out = Vec::<String>::new();
    for val in varray {
        out.push(try!(v.as_str().ok_or(Error::DataParseError("Not a string".into()))).into());
    }

    Ok(out)
}

fn parse_server_entry(entry: &QStatServer) -> Result<ServerEntry, Error> {
    let mut v = ServerEntry::default();
    v.host = try!(entry.address
        .clone()
        .ok_or(Error::NullException("Host address cannot be empty".into())));

    Ok(v)
}

fn parse(raw: &str, server_type: &str) -> Result<ServerData, Error> {
    let data: QStatResponse = try!(serde_json::from_str(raw));

    Ok(data.iter().fold(ServerData::default(), |mut acc, ref entry| {
        match parse_server_entry(&entry) {
            Ok(v) => {
                acc.insert(v);
            }
            _ => {}
        }
        acc
    }))
}

pub fn query(settings: &ConfStorage) -> Result<ServerData, Error> {
    let qstat_path: String = try!(settings.get_or_err("qstat_path"));
    let master_type: String = try!(settings.get_or_err("qstat_master_type"));
    let server_type: String = try!(settings.get_or_err("qstat_server_type"));
    let master_server_uri: Vec<String> = try!(settings.get_or_err("master_server_uri"));

    let mut rules = HashMap::<String, String>::new();
    match settings.get("qstat_game_type".into()) {
        Some(val) => {
            match *val {
                Value::String(ref v) => {
                    rules.insert("qstat_game_type".into(), v.clone());
                }
                _ => {}
            }
        }
        _ => {}
    }

    let mut child = try!(Command::new(qstat_path)
        .arg(make_qstat_cmd_params(&master_type, &rules, &master_server_uri).join(" "))
        .stdout(Stdio::piped())
        .spawn());

    let out = try!(child.wait_with_output());
    if !out.status.success() {
        return Err(Error::IOError(match out.status.code() {
            Some(code) => format!("QStat exited with status: {}", code).into(),
            None => "QStat killed by a signal".into(),
        }));
    }

    parse(try!(String::from_utf8(out.stdout.clone())).as_str(),
          &server_type)
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
        let expectation: Vec<String> = vec![String::from("-json"),
                                            String::from("-utf8"),
                                            String::from("-maxsim"),
                                            String::from("9999"),
                                            String::from("-R"),
                                            String::from("-P"),
                                            String::from("-q3s") + &make_rulestring(&rules),
                                            String::from("serverA serverB")];
        assert_eq!(func("q3s", &rules, &master_server_uri), expectation);
    }

    #[test]
    fn test_parse() {
        let fixture = r##"
[
    {
        "protocol": "a2s",
        "address": "123.456.789.0:33333",
        "status": "online",
        "hostname": "123.456.789.0:33333",
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
        expectation.insert(ServerEntry::default());
        assert_eq!(func(fixture, "a2s").unwrap(), expectation);
    }
}
