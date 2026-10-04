use ureq::Agent;
use roxmltree::Document;
use directories::ProjectDirs;
use std::time::Duration;
use std::fs::OpenOptions;
use std::fs;
use std::io;
use std::io::{BufRead,Write};
use std::path::Path;


fn fetch(uri: String,agent: Agent) -> Option<String> {
    agent.get(uri).call().ok()?.body_mut().read_to_string().ok()
}

fn parse_videos(xml: &Document, seen: &mut io::BufReader<&std::fs::File>, writer: &mut io::BufWriter<&std::fs::File>) {
    for entry in xml.descendants().filter(|n| n.tag_name().name() == "entry") {
        if let Some(id) = entry.children().find(|n| n.tag_name().name() == "id") {
            let found = id.text().unwrap();
            if !seen.lines().any(|line| line.unwrap() == found) {
                println!("{}",found);

                writeln!(writer,"{}",found).ok();
            }
        }
    }
    writer.flush().ok();
}

fn run(dir: &Path, agent: Agent, channel_id: String) -> Option<()> { // this is probably supposed to be a Result but like idk how to use it properly yet
    fs::create_dir_all(dir.join("seen")).ok()?;
    let seen_path = dir.join(format!("seen/{channel_id}.txt"));
    let seen = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .open(seen_path)
        .unwrap();

    let mut reader = io::BufReader::new(&seen);
    let mut writer = io::BufWriter::new(&seen);

    println!("{}",dir.to_str()?);

    if let Some(response) = fetch(format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}"), agent) {
        let xml = &response;
        println!("gotcha");
        if let Ok(res) = Document::parse(xml) {
            parse_videos(&res, &mut reader, &mut writer);
        }
    }
    None
}

fn main() {
    let config = Agent::config_builder()
        .user_agent("guhw-yt/1.0")
        .timeout_global(Some(Duration::from_secs(5)))
        .build();

    let agent: Agent = config.into();

    if let Some(dir) = ProjectDirs::from("dev", "guhw", "yt-notify") {
        run(dir.config_dir(), agent, "UCuLpYAaxNfBdODBRd8pOfXQ".to_string());
    }
}
