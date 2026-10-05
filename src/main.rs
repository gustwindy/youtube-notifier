use roxmltree::{Document, Node};
use std::collections::HashSet;
use notify_rust::Notification;
use directories::ProjectDirs;
use std::io::{BufRead,Write};
use std::process::Command;
use std::fs::OpenOptions;
use std::time::Duration;
use serde::{Serialize};
use std::path::Path;
use ureq::Agent;
use std::fs;
use std::io;

struct Args {
    yt_channel_id: String,
    webhook: Option<String>,
    user_id: Option<String>,
    should_open_in_browser: bool,
    helped: bool,
    verbose: bool,
    super_verbose: bool,
}

fn get_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut sent = Args {
        yt_channel_id: "".to_string(),
        webhook: None,
        user_id: None,
        should_open_in_browser: false,
        helped: false,
        verbose: false,
        super_verbose: false
    };

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('w') | Long("webhook-url") => {
                sent.webhook = Some(parser.value()?.string()?);
            }
            Short('u') | Long("user-id") => {
                sent.user_id = Some(parser.value()?.string()?);
            }
            Short('b') | Long("open-browser") => {
                sent.should_open_in_browser = true;
            }
            Short('+') | Long("verbose+") => {
                sent.verbose = true;
                sent.super_verbose = true;
            }
            Short('v') | Long("verbose") => {
                sent.verbose = true;
            }
            Value(val) => {
                sent.yt_channel_id = val.string()?;
            }
            Short('h') | Long("help") => {
                println!("Usage: yt-notify [-w|--webhook-url=DISCORD_URL] [-u|--user-id=DISCORD_ID] [-b|--open-browser] [-v|--verbose] [-+|--verbose+] <YOUTUBE_CHANNEL_ID>");
                sent.helped = true;
            }
            _ => return Err(arg.unexpected())
        }
    }

    Ok(sent)
}

fn fetch(uri: String,agent: &Agent) -> Option<String> {
    agent.get(uri).call().ok()?.body_mut().read_to_string().ok()
}

fn get_first<'a>(node: &'a Node<'a, 'a>, tag: &str) -> Option<Node<'a, 'a>> { // copied from the error i have no clue what that means
    node.children().find(|n| n.tag_name().name() == tag)
}

#[derive(Serialize)]
struct Webhook {
    content: String,
}

fn iterate_videos(agent: &Agent, args: Args, xml: &Document, seen: &mut io::BufReader<&std::fs::File>, writer: &mut io::BufWriter<&std::fs::File>) {
    let seen_ids: HashSet<String> = seen
        .lines()
        .filter_map(Result::ok)
        .collect();

    for entry in xml.descendants().filter(|n| n.tag_name().name() == "entry") {
        let Some(id) = get_first(&entry,"videoId") else {continue};
        let id = id.text().unwrap();

        if !seen_ids.iter().any(|seen_id| seen_id == id) {
            let Some(title) = get_first(&entry,"title") else {continue};
            let title = title.text().unwrap();
            let url = format!("https://www.youtube.com/watch?v={id}");

            if args.should_open_in_browser {
                Command::new("xdg-open").arg(&url).output().ok();
            }
            if let Some(webhook) = &args.webhook {
                let mut content = "0";
                if let Some(user) = &args.user_id {
                    content = user;
                }
                agent.post(webhook).send_json(&Webhook {
                    content: format!("new video <@{content}> {url}")
                }).ok();
            }

            Notification::new()
                .summary(title)
                .show().ok();
            writeln!(writer,"{}",id).ok();
        }
    }
    writer.flush().ok();
}

fn run(dir: &Path, agent: Agent, args: Args) -> Option<()> { // this is probably supposed to be a Result but like idk how to use it here
    fs::create_dir_all(dir.join("seen")).ok()?;
    let channel_id = &args.yt_channel_id;

    let seen_path = dir.join(format!("seen/{channel_id}.txt"));
    let seen = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(&seen_path)
        .unwrap();

    let mut reader = io::BufReader::new(&seen);
    let mut writer = io::BufWriter::new(&seen);

    if args.verbose {
        println!("{}",dir.to_str()?);
        println!("{}",seen_path.to_str()?);
    }

    if let Some(response) = fetch(format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"), &agent) {
        let xml = &response;
        if args.verbose {
            println!("got response");
        }
        if args.super_verbose {
            println!("{}", xml);
        }
        if let Ok(res) = Document::parse(xml) {
            iterate_videos(&agent, args, &res, &mut reader, &mut writer);
        }
    }
    None
}

fn main() -> Result<(),lexopt::Error> {
    let args: Args = get_args()?;
    if args.helped {
        return Ok(());
    }

    if args.yt_channel_id == "".to_string() {
        println!("you might want to --help");
        return Ok(());
    }
    let config = Agent::config_builder()
        .user_agent("guhw-yt/1.0")
        .timeout_global(Some(Duration::from_secs(5)))
        .build();

    let agent: Agent = config.into();

    if let Some(dir) = ProjectDirs::from("dev", "guhw", "yt-notify") {
        run(dir.config_dir(), agent, args);
    }

    Ok(())
}
