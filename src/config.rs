// serde-based json config
use serde_derive::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub title : String,
    pub banner : String,
    pub icon : String,
    pub database : String,
    pub images : String, // image dir
    pub new_user_token : Option<String> // if this board doesn't allow public user creation...
}


pub fn get_config() -> Config {
    let cnf : Config = serde_json::from_reader(std::fs::File::open("config.json").unwrap()).unwrap();
    std::fs::create_dir_all(&cnf.images).unwrap();
    cnf
}
