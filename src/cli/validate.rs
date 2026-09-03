use crate::error::RsyncResult;
use crate::persistance::config::Config;
use crate::r2::client::R2Client;

pub fn run() -> RsyncResult<()> {
    let config = Config::load()?;

    R2Client::new(&config).validate()?;

    println!("bucket '{}' is reachable", config.settings.bucket);
    Ok(())
}
