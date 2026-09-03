use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::*;

const STATE_FILE: &str = ".state.toml";

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    files: BTreeMap<String, String>,
}

impl State {
    pub fn load() -> RsyncResult<Self> {
        let path = Path::new(STATE_FILE);

        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(path)
            .map_err(|err| Error::Persistance(format!("cannot read {STATE_FILE}: {err}")))?;

        toml::from_str(&raw)
            .map_err(|err| Error::Persistance(format!("cannot parse {STATE_FILE}: {err}")))
    }

    pub fn is_pushed(&self, key: &str, hash: &str) -> bool {
        self.files.get(key).is_some_and(|value| value == hash)
    }

    pub fn push(&mut self, key: String, hash: String) {
        self.files.insert(key, hash);
    }

    pub fn save(&self) -> RsyncResult<()> {
        let body = toml::to_string_pretty(self)
            .map_err(|err| Error::Persistance(format!("cannot serialize {STATE_FILE}: {err}")))?;

        std::fs::write(STATE_FILE, body)
            .map_err(|err| Error::Persistance(format!("cannot write {STATE_FILE}: {err}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml() {
        let mut state = State::default();
        state.push("images/abc.png".into(), "123456".into());

        let raw = toml::to_string_pretty(&state).unwrap();
        let restored: State = toml::from_str(&raw).unwrap();

        assert!(restored.is_pushed("images/abc.png", "123456"));
        assert!(!restored.is_pushed("images/abc.png", "000000"));
        assert!(!restored.is_pushed("images/other.png", "123456"));
    }
}
