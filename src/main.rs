//! mastodon — post + read via the Mastodon REST API.
//! Config: ~/.gray/mastodon/config.json {"instance","token"} or env
//! MASTODON_INSTANCE / MASTODON_TOKEN.

use serde_json::{json, Value};
use std::io::{BufRead, Write};

const NAME: &str = "mastodon";

fn manifest() -> Value {
    json!({
        "name": NAME, "display_name": "Mastodon",
        "version": env!("CARGO_PKG_VERSION"), "protocol": "1.1",
        "tools": [
            {"name": "mastodon_post",
             "description": "Post a status. visibility: public|unlisted|private|direct (default unlisted).",
             "parameters": {"type": "object", "properties": {
                "text": {"type": "string"}, "visibility": {"type": "string"},
                "reply_to": {"type": "string", "description": "status id to reply to"},
                "spoiler": {"type": "string"}}, "required": ["text"]}},
            {"name": "mastodon_notifications",
             "description": "Recent notifications (mentions, boosts, favs, follows).",
             "parameters": {"type": "object", "properties": {
                "limit": {"type": "integer"}}}},
            {"name": "mastodon_timeline",
             "description": "Read a timeline: home|local|public (default home).",
             "parameters": {"type": "object", "properties": {
                "scope": {"type": "string"}, "limit": {"type": "integer"}}}},
            {"name": "mastodon_search",
             "description": "Search statuses/accounts/hashtags.",
             "parameters": {"type": "object", "properties": {
                "query": {"type": "string"}, "limit": {"type": "integer"}},
             "required": ["query"]}},
        ],
        "commands": ["/mastodon"], "subcommands": ["mastodon"],
    })
}

fn config_path() -> std::path::PathBuf {
    let home = std::env::var_os("GRAY_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".gray")))
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|h| std::path::PathBuf::from(h).join(".gray"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join("mastodon").join("config.json")
}
fn read_config() -> Value {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}))
}
fn cfg(k: &str, env: &str) -> Option<String> {
    std::env::var(env)
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| read_config()[k].as_str().map(str::to_string))
}

fn api(method: &str, path: &str, body: Option<&Value>) -> Result<Value, String> {
    let inst = cfg("instance", "MASTODON_INSTANCE")
        .ok_or("no instance — `mastodon setup --instance mastodon.social --token …`")?;
    let tok = cfg("token", "MASTODON_TOKEN").ok_or("no token configured")?;
    let url = format!("{}/api/v1{}", inst.trim_end_matches('/'), path);
    let req = match method {
        "GET" => ureq::get(&url),
        _ => ureq::post(&url),
    }
    .timeout(std::time::Duration::from_secs(20))
    .set("Authorization", &format!("Bearer {tok}"));
    let resp = match body {
        Some(b) => req.set("Content-Type", "application/json").send_json(b),
        None => req.call(),
    };
    match resp {
        Ok(r) => r.into_json().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(c, r)) => Err(format!(
            "HTTP {c}: {}",
            r.into_string()
                .unwrap_or_default()
                .chars()
                .take(250)
                .collect::<String>()
        )),
        Err(e) => Err(e.to_string()),
    }
}

fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut tag = false;
    for c in s.chars() {
        match c {
            '<' => tag = true,
            '>' => tag = false,
            _ if !tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fmt_status(s: &Value) -> String {
    format!(
        "@{} · {}\n  {}",
        s["account"]["acct"].as_str().unwrap_or("?"),
        s["created_at"].as_str().unwrap_or(""),
        strip_html(s["content"].as_str().unwrap_or(""))
            .chars()
            .take(200)
            .collect::<String>()
    )
}

fn tool(name: &str, args: &Value) -> Value {
    let r = match name {
        "mastodon_post" => {
            let mut b = json!({"status": args["text"].as_str().unwrap_or(""),
                "visibility": args["visibility"].as_str().unwrap_or("unlisted")});
            if let Some(r) = args["reply_to"].as_str() {
                b["in_reply_to_id"] = json!(r);
            }
            if let Some(s) = args["spoiler"].as_str() {
                b["spoiler_text"] = json!(s);
            }
            api("POST", "/statuses", Some(&b))
                .map(|v| format!("posted → {}", v["url"].as_str().unwrap_or("(no url)")))
        }
        "mastodon_notifications" => {
            let n = args["limit"].as_u64().unwrap_or(15).min(40);
            api("GET", &format!("/notifications?limit={n}"), None).map(|v| {
                v.as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(|n| {
                        format!(
                            "[{}] @{} {}",
                            n["type"].as_str().unwrap_or("?"),
                            n["account"]["acct"].as_str().unwrap_or("?"),
                            n["status"]["content"]
                                .as_str()
                                .map(|c| strip_html(c).chars().take(120).collect::<String>())
                                .unwrap_or_default()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        "mastodon_timeline" => {
            let scope = args["scope"].as_str().unwrap_or("home");
            let n = args["limit"].as_u64().unwrap_or(15).min(40);
            let path = match scope {
                "local" => "/timelines/public?local=true",
                "public" => "/timelines/public",
                _ => "/timelines/home",
            };
            api("GET", &format!("{path}&limit={n}"), None).map(|v| {
                v.as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(fmt_status)
                    .collect::<Vec<_>>()
                    .join("\n\n")
            })
        }
        "mastodon_search" => {
            let q = args["query"].as_str().unwrap_or("");
            let n = args["limit"].as_u64().unwrap_or(10).min(40);
            api(
                "GET",
                &format!(
                    "/search?q={}&limit={n}&resolve=true",
                    q.chars()
                        .map(|c| if c.is_alphanumeric() {
                            c.to_string()
                        } else {
                            format!("%{:02X}", c as u32)
                        })
                        .collect::<String>()
                ),
                None,
            )
            .map(|v| {
                v["statuses"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(fmt_status)
                    .collect::<Vec<_>>()
                    .join("\n\n")
            })
        }
        t => Err(format!("unknown tool {t}")),
    };
    match r {
        Ok(c) => json!({"result": {"content": if c.is_empty() {"(empty)".into()} else {c}}}),
        Err(e) => json!({"result": {"content": e, "is_error": true}}),
    }
}

fn cli(argv: &[String]) -> Result<String, String> {
    match argv.first().map(String::as_str) {
        Some("setup") => {
            let mut c = read_config();
            let mut it = argv[1..].iter();
            while let Some(f) = it.next() {
                match f.as_str() {
                    "--instance" => {
                        c["instance"] = json!(it.next().ok_or("--instance needs host")?)
                    }
                    "--token" => c["token"] = json!(it.next().ok_or("--token needs value")?),
                    o => return Err(format!("unknown flag {o}")),
                }
            }
            std::fs::create_dir_all(config_path().parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(config_path(), serde_json::to_string_pretty(&c).unwrap())
                .map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    std::fs::set_permissions(config_path(), std::fs::Permissions::from_mode(0o600));
            }
            Ok("saved".into())
        }
        Some("status") | None => Ok(format!(
            "Mastodon\n  instance: {}\n  token: {}",
            cfg("instance", "MASTODON_INSTANCE").unwrap_or("MISSING".into()),
            if cfg("token", "MASTODON_TOKEN").is_some() {
                "set"
            } else {
                "MISSING"
            }
        )),
        Some("-h" | "--help" | "help") => Ok("usage: mastodon setup | status".into()),
        Some(o) => Err(format!("unknown verb {o}")),
    }
}

fn sidecar() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let method = v["method"].as_str().unwrap_or("");
        match v.get("id").cloned() {
            None if method == "plugin/shutdown" => return Ok(()),
            None => {}
            Some(id) => {
                let reply = if method == "plugin/manifest" {
                    json!({"id": id, "result": manifest()})
                } else if method == "tool/call" {
                    let mut r = tool(
                        v["params"]["name"].as_str().unwrap_or(""),
                        &v["params"]["args"],
                    );
                    r["id"] = id;
                    r
                } else if method == "command/run" {
                    let argv: Vec<String> = v["params"]["argv"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    let text = cli(&argv).unwrap_or_else(|e| e);
                    json!({"id": id, "result": {"text": text}})
                } else {
                    json!({"id": id, "error": {"code": -32601,
                        "message": format!("method not found: {method}")}})
                };
                writeln!(out, "{reply}")?;
                out.flush()?;
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<String> = match args.first().map(String::as_str) {
        Some("mastodon") => args[1..].to_vec(),
        _ => args,
    };
    match args.first().map(String::as_str) {
        Some("manifest") => println!("{}", manifest()),
        Some("sidecar") => {
            let _ = sidecar();
        }
        _ => {
            if args.is_empty() {
                if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                    println!("mastodon — `mastodon setup --instance https://x.social --token …`");
                    return;
                }
                let _ = sidecar();
                return;
            }
            match cli(&args) {
                Ok(t) => println!("{t}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
