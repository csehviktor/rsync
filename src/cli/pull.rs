use std::collections::BTreeSet;
use std::path::Path;

use crate::error::*;
use crate::persistance::config::*;
use crate::persistance::state::*;
use crate::r2::client::*;
use crate::r2::sigv4;

pub fn run() -> RsyncResult<()> {
    let config = Config::load()?;
    let mut state = State::load()?;

    let client = R2Client::new(&config);
    let bucket = &config.settings.bucket;

    println!("pulling changes from '{bucket}'...");

    let remote: BTreeSet<String> = client.list()?.into_iter().collect();

    let deleted: Vec<String> = state
        .keys()
        .filter(|key| !remote.contains(*key))
        .cloned()
        .collect();

    let pending: Vec<String> = remote
        .into_iter()
        .filter(|key| !config.is_excluded(key))
        .filter(|key| !Path::new(key).exists())
        .collect();

    let mut changes = 0;

    for key in pending {
        let content = client.get(&key)?;

        if let Some(dir) = Path::new(&key).parent() {
            std::fs::create_dir_all(dir)?;
        }

        std::fs::write(&key, &content)?;
        println!("+ pulled {key}");

        state.push(key, sigv4::hash(&content));
        changes += 1;
    }

    for key in deleted {
        // the file may already be gone if it was deleted on both sides
        if Path::new(&key).exists() {
            std::fs::remove_file(&key)?;
            println!("- deleted {key}");
        }

        state.remove(&key);
    }

    println!("pulled {changes} change(s) from '{bucket}'");
    state.save()
}
