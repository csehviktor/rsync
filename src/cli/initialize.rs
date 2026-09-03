use std::path::Path;

use crate::error::*;
use crate::persistance::config::*;
use crate::persistance::state::*;

pub fn run(bucket: &str) -> RsyncResult<()> {
    if Path::new(CONFIG_FILE).exists() || Path::new(STATE_FILE).exists() {
        return Err(Error::Persistance(
            "this directory is already initialized".into(),
        ));
    }

    std::fs::write(CONFIG_FILE, Config::default(bucket))?;
    State::default().save()?;

    println!("initialized '{bucket}', fill in '{CONFIG_FILE}' with your r2 credentials");
    Ok(())
}
