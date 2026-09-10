use std::path::Path;

use crate::error::*;
use crate::persistance::config::*;
use crate::persistance::state::*;
use crate::r2::client::*;
use crate::r2::sigv4;
use crate::walk;

pub fn run() -> RsyncResult<()> {
    let config = Config::load()?;
    let mut state = State::load()?;

    let client = R2Client::new(&config);
    let bucket = &config.settings.bucket;

    println!("pushing changes from '{bucket}'...");

    let mut changes = 0;

    for key in walk::collect_files(Path::new("."), |key| config.is_excluded(key))? {
        let content = std::fs::read(&key)?;
        let hash = sigv4::hash(&content);

        if state.is_pushed(&key, &hash) {
            continue;
        }

        client.put(&key, content)?;
        println!("+ pushed {key}");

        state.push(key, hash);
        changes += 1;
    }

    let deleted: Vec<String> = state
        .keys()
        .filter(|key| !Path::new(key).exists())
        .cloned()
        .collect();

    for key in deleted {
        client.delete(&key)?;
        println!("- deleted {key}");

        state.remove(&key);
        changes += 1;
    }

    println!("pushed {changes} change(s) to '{bucket}'");
    state.save()
}
