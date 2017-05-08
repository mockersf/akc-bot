extern crate logger;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;

extern crate iron;
extern crate router;
extern crate urlencoded;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate bodyparser;

extern crate hyper;
extern crate hyper_native_tls;
extern crate futures;
extern crate futures_cpupool;

extern crate uuid;
extern crate ini;
#[macro_use]
extern crate lazy_static;

mod log_message;
mod middlewares;
mod handlers;
mod clients;
mod sami;

use std::env;
use iron::prelude::Chain;
use iron::Iron;
use router::Router;
use logger::Logger;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use futures_cpupool::CpuPool;
use ini::Ini;

struct Configuration {
    witai_token: String,
    witai_version: String,
    akc_appid: String,
    akc_appsecret: String,
    hipchat_command: String,
}

lazy_static! {
    static ref CONFIGURATION: Configuration = {
        info!("reading configuration");
        let conf = Ini::load_from_file("conf.ini").unwrap();

        let witai_section = conf.section(Some("WitAI".to_owned())).unwrap();
        let witai_token = witai_section.get("token").unwrap();
        let witai_version = witai_section.get("version").unwrap();

        let akc_section = conf.section(Some("AKC".to_owned())).unwrap();
        let akc_appid = akc_section.get("appId").unwrap();
        let akc_appsecret = akc_section.get("appSecret").unwrap();

        let hipchat_section = conf.section(Some("HipChat".to_owned())).unwrap();
        let hipchat_command = hipchat_section.get("command").unwrap();
        Configuration {
            witai_token: witai_token.to_owned(),
            witai_version: witai_version.to_owned(),
            akc_appid: akc_appid.to_owned(),
            akc_appsecret: akc_appsecret.to_owned(),
            hipchat_command: hipchat_command.to_owned(),
        }
    };
}

lazy_static! {
    static ref REQUEST_CPU_POOL: CpuPool = {
        CpuPool::new_num_cpus()
    };
}

#[derive(Clone, Debug)]
pub struct Database {
    tokens: HashMap<String, ::clients::oauth2::Token>,
}
impl Database {
    pub fn new() -> Database {
        Database { tokens: HashMap::new() }
    }
    pub fn add_token(&mut self, from: String, token: ::clients::oauth2::Token) {
        info!("setting token {} - {:?}", from, token);
        self.tokens.insert(from, token);
    }
    pub fn get_token(&self, key: String) -> Option<&::clients::oauth2::Token> {
        self.tokens.get(&key)
    }
    pub fn remove_token(&mut self, access_token: String) {
        let keys_to_remove: Vec<String> = self.tokens.clone().iter().filter(|&(_, v)| v.access_token() == access_token).map(|(k, _)| k.clone()).collect();
        for key in keys_to_remove {
            self.tokens.remove(&key);
        }
    }
}
lazy_static! {
    static ref DATABASE: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database::new()));
}

fn main() {
    let format = |record: &LogRecord| {
        let t = time::now();
        format!("{},{:03} - {} - {}: {}",
                time::strftime("%Y-%m-%d %H:%M:%S", &t).unwrap(),
                t.tm_nsec / 1000_000,
                record.level(),
                record.location().module_path(),
                record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Info);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init().unwrap();

    info!("application starting");

    let (logger_before, logger_after) = Logger::new(None);

    let mut router = Router::new();
    router.post("/hipchat/notification",
                handlers::hipchat::ReceiveNotification::new(),
                "hipchat_notification");

    router.get("/akc/auth",
               handlers::akc::ExchangeToken::new(),
               "akc_exchange_token");

    router.post("/test/:from",
                handlers::test::SetTokenForContext::new(),
                "set_token");
    router.get("/test/:from/user",
               handlers::test::GetUserFromContext::new(),
               "get_user_for_token");
    router.get("/test/:from/devices",
               handlers::test::GetDevicesFromContext::new(),
               "get_devices_for_token");
    router.get("/test/:from/devicetypes",
               handlers::test::GetDeviceTypesFromContext::new(),
               "get_devicetypes_for_token");
    router.get("/test/:from/snapshot/:sdid",
               handlers::test::GetSnapshotFromContext::new(),
               "get_snapshot_for_device_for_token");



    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(middlewares::Default404);
    chain.link_after(middlewares::JsonResponse);
    chain.link_after(middlewares::ErrorLogger);
    chain.link_after(logger_after);


    Iron::new(chain).http("localhost:3000").unwrap();

}
