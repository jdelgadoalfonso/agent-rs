use dirs;

use ext_config::{Config, File};

use rustls::ClientConfig;

use std::{fs, io::BufReader};


#[derive(Clone)]
pub struct Configuration {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub vhost: String,
    pub cafile: String,
}

pub fn config() -> Configuration {
    let mut home = dirs::home_dir().unwrap();
    home.push(".agent-rs");

    let mut settings = Config::default();

    settings.merge(File::from(home)).unwrap();

    Configuration {
        username: settings.get::<String>("username").unwrap(),
        password: settings.get::<String>("password").unwrap(),
        host: settings.get::<String>("host").unwrap(),
        port: settings.get::<u16>("port").unwrap_or(5671),
        vhost: settings.get::<String>("vhost").unwrap_or(String::from("/")),
        cafile: settings.get::<String>("cafile").unwrap(),
    }
}

pub fn client_config_with_root_ca(cafile: &String) -> ClientConfig {
    let mut client_config = ClientConfig::new();
    let certfile = fs::File::open(cafile).expect("Cannot open CA file");
    let mut reader = BufReader::new(certfile);

    client_config.root_store.add_pem_file(&mut reader).unwrap();

    client_config
}