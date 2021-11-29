mod routes;

use rocket::config::Config;
use rocket::log::LogLevel;
use rocket::{catchers, routes, Build, Rocket};
use rocket_slogger::{o, Drain, Logger, Slogger};
use routes::{always_fail, always_greet, always_thank, not_found};

use std::sync::Mutex;

#[rocket::launch]
async fn rocket() -> Rocket<Build> {
    let bunyan_logger = slog_bunyan::with_name("My App", std::io::stderr()).build();
    let logger = Logger::root(Mutex::new(bunyan_logger).fuse(), o!());

    let fairing = Slogger::from_logger(logger);

    let mut config = Config::default();
    config.log_level = LogLevel::Off;

    rocket::custom(config)
        .attach(fairing)
        .mount("/", routes![always_greet, always_thank, always_fail])
        .register("/", catchers![not_found])
}
