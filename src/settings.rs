use std::env;

use std::path::PathBuf;
use config::{ConfigError, Config, File, Environment};
use directories::ProjectDirs;
use serde_derive::Deserialize;

/// Configuration for the server.
/// These values are set when the server initializes, and do not change while running.
/// These are constructed from default or local files and ENV variables. 
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    /// The address to listen on
    pub server_url: String,
    pub redis_url: String,
    pub redis_cluster_url: String,
    pub config_dir: PathBuf,
    pub api_endpoint: String,
}
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {

    let mut settings = Config::builder();

    // Start off by merging in the "default" configuration file
    // settings.merge(File::with_name("config/default"))?;
    let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
    println!("env: {}", env);
    if let Some(proj_dirs) = ProjectDirs::from("com", "aks",  "terraphim") {
        let config_dir=proj_dirs.config_dir();
        println!("Project Dir {:?}", config_dir);
        settings = settings.set_default("config_dir", config_dir.to_str())?;
        println!("Create folder if doesn't exist");
        std::fs::create_dir_all(proj_dirs.config_dir()).unwrap();
        let filename= proj_dirs.config_dir().join("config.toml");
        
        if filename.exists() {
            println!("File exists");
            println!("{:?}", filename);
        } else {
            println!("File does not exist");
            std::fs::copy("config/default.toml", &filename).unwrap();
        }
        
        settings=settings.add_source(File::with_name(filename.to_str().unwrap()));  
        

    }

    // settings.merge(File::with_name(".env"))?;
    settings=settings.add_source(Environment::with_prefix("TERRAPHIM"));
    match settings.build() {
        Ok(config) => {
            println!("Settings: {:?}", config);
            Ok(config.try_deserialize())?
        },
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }

    }
}