use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    /// Path to the folder containing the mod jar's
    pub target_path: PathBuf,

    /// Minecraft Server version as a string
    pub server_version: String,

    /// Mod Loader that is being used as a string
    pub loader_version: String,

    pub backup_mods: bool,

    pub backup_path: PathBuf,

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
            backup_mods: true,
            backup_path: Path::new(".backup").into(),
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
        fs::rename(out_path, &self.path)?;

        Ok(())
    }
}
