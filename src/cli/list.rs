use std::cmp::Ordering;
use std::path::Path;

use crate::error::*;
use crate::persistance::config::Config;
use crate::r2::client::R2Client;
use crate::walk::collect_files;

const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

const REMOTE_ONLY: &str = "remote";
const LOCAL_ONLY: &str = "local";
const SYNCED: &str = "synced";

pub fn run() -> RsyncResult<()> {
    let config = Config::load()?;
    let client = R2Client::new(&config);

    let local = collect_files(Path::new("."))?;
    let remote = client.list()?;

    merge(&local, &remote);

    Ok(())
}

fn merge(local: &[String], remote: &[String]) {
    let (mut i, mut j) = (0, 0);

    loop {
        let order = match (local.get(i), remote.get(j)) {
            (Some(l), Some(r)) => l.cmp(r),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => break,
        };

        match order {
            Ordering::Less => {
                line(RED, LOCAL_ONLY, &local[i]);
                i += 1;
            }
            Ordering::Equal => {
                line(GREEN, SYNCED, &local[i]);
                i += 1;
                j += 1;
            }
            Ordering::Greater => {
                line(YELLOW, REMOTE_ONLY, &remote[j]);
                j += 1;
            }
        }
    }
}

#[inline]
fn line(color: &str, indicator: &str, key: &str) {
    println!("{color}{indicator} {RESET}{key}");
}
