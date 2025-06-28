use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};

/// A response to the client from the server
#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct Config {
    /// Maximum filesize in bytes
    pub target_path: PathBuf,

    /// Maximum filesize in bytes
    pub server_version: String,

    /// Is overwiting already uploaded files with the same hash allowed, or is

    pub loader_version: String,

    #[serde(skip)]
    path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_path: Path::new(".").into(),
            server_version: String::new(),
            loader_version: String::new(),
            path: "./settings.toml".into(),
        }
    }
}

impl Config {
    pub fn open<P: AsRef<Path>>(path: &P) -> Result<Self, io::Error> {
        let mut input_str = String::new();
        if !path.as_ref().exists() {
            let new_self = Self {
                path: path.as_ref().to_path_buf(),
                ..Default::default()
            };
            new_self.save()?;
            return Ok(new_self);
        } else {
            File::open(path).unwrap().read_to_string(&mut input_str)?;
        }

        let mut parsed_config: Self = toml::from_str(&input_str).unwrap();
        parsed_config.path = path.as_ref().to_path_buf();

        Ok(parsed_config)
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let out_path = &self.path.with_extension("new");
        let mut file = File::create(out_path)?;
        file.write_all(&toml::to_string_pretty(self).unwrap().into_bytes())?;

        // Overwrite the original DB with
        fs::rename(out_path, &self.path).unwrap();

        Ok(())
    }
}