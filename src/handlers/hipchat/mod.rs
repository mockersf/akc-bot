//use std::collections::Vec;
use std::sync::{Arc, Mutex};

pub mod descriptor;
pub mod notification;
pub mod installation_callback;

#[derive(Clone, Debug)]
pub struct Database {
    installations: Vec<::clients::hipchat::installation::Installation>,
}
impl Database {
    pub fn new() -> Database {
        Database { installations: vec![] }
    }
    pub fn add_installation(&mut self, installation: ::clients::hipchat::installation::Installation) {
        info!("adding installation {:?}", installation);
        self.installations.push(installation);
    }
    pub fn get_installation(&self, key: String) -> Option<&::clients::hipchat::installation::Installation> {
        info!("getting installation for key {:?}", key);
        self.installations
            .iter()
            .find(|installation| installation.oauth_id == key)
    }
    /*    pub fn remove_token(&mut self, access_token: String) {
        let keys_to_remove: Vec<String> = self.installations
            .clone()
            .iter()
            .filter(|&(_, v)| v.access_token() == access_token)
            .map(|(k, _)| k.clone())
            .collect();
        for key in keys_to_remove {
            self.installations.remove(&key);
        }
    }*/
}
lazy_static! {
    static ref HC_DATABASE: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database::new()));
}
