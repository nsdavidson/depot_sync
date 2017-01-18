use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use origin::Origin;
use serde_yaml;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub origins: Vec<Origin>,
    pub source: String,
    pub destination: String,
    pub token: String,
}

impl Config {
    pub fn from_file(path: String) -> Result<Config, &'static str> {
        let config_path = Path::new(&path);
        let mut config_file = match File::open(config_path) {
            Ok(file) => file,
            Err(e) => panic!("couldn't open file {}: {}", path, e.description()),
        };

        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).expect("couldn't read file to string");
        let config: Config = serde_yaml::from_str(&config_string).unwrap();
        Ok(config)
    }
}
