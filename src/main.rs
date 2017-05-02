extern crate logger;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate time;

extern crate iron;
extern crate router;

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

mod middlewares;
mod handlers;
mod clients;

use std::env;
use iron::prelude::Chain;
use iron::Iron;
use router::Router;
use logger::Logger;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

use futures_cpupool::CpuPool;

use ini::Ini;

struct Configuration {
    wit_ai_token: String,
    version: String,
}

lazy_static! {
    static ref CONFIGURATION: Configuration = {
        info!("reading configuration");
        let conf = Ini::load_from_file("conf.ini").unwrap();
        let section = conf.section(Some("WitAI".to_owned())).unwrap();
        let token = section.get("token").unwrap();
        let version = section.get("version").unwrap();
        Configuration {
            wit_ai_token: token.to_owned(),
            version: version.to_owned(),
        }
    };
}

lazy_static! {
    static ref REQUEST_CPU_POOL: CpuPool = {
        CpuPool::new_num_cpus()
    };
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

    let mut chain = Chain::new(router);
    chain.link_before(logger_before);
    chain.link_after(middlewares::Default404);
    chain.link_after(middlewares::JsonResponse);
    chain.link_after(middlewares::ErrorLogger);
    chain.link_after(logger_after);


    Iron::new(chain).http("localhost:3000").unwrap();

}
