use std::{
    error::Error,
    fs::{self, create_dir_all, File},
    io::{BufReader, BufWriter, Read, Write},
};

use serde::Deserialize;

use crate::files::get_home_dir;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default_browse: Option<bool>,
    pub custom_output: Option<String>,
}

pub fn create_default_config_file() -> Result<bool, Box<dyn Error>> {
    let home_dir = get_home_dir();
    let path = home_dir.join(".config/iconify-rs/");

    create_dir_all(&path.as_path())?;

    let default_config = include_str!("../config-default.toml");

    let file = File::create(path.join("iconify-rs.toml"))?;
    let mut writer = BufWriter::new(file);

    writer.write_all(default_config.as_bytes())?;

    Ok(true)
}

pub fn read_config_file() -> Result<Config, Box<dyn Error>> {
    let file_path = get_home_dir().join(".config/iconify-rs/iconify-rs.toml");

    if !fs::metadata(file_path.clone()).is_ok() {
        create_default_config_file()?;
    }

    let file = File::open(file_path)
        .expect("config file should be present at ~/.config/iconify-rs/iconify-rs.toml");
    let mut reader = BufReader::new(file);
    let mut result = String::new();

    reader.read_to_string(&mut result)?;

    let config = toml::from_str(&result)?;

    Ok(config)
}
