mod routes;

use rocket::config::Config;
use rocket::log::LogLevel;
use rocket::{catchers, routes, Build, Rocket};
use rocket_slogger::{o, Drain, Logger, Slogger};
use routes::{always_fail, always_greet, always_thank, dynamic_path, not_found};

use std::sync::Mutex;

#[rocket::launch]
async fn rocket() -> Rocket<Build> {
    let bunyan_logger = slog_bunyan::with_name("My App", std::io::stderr()).build();
    let logger = Logger::root(Mutex::new(bunyan_logger).fuse(), o!());

    let fairing = Slogger::from_logger(logger);

    // Turn off Rocket logging, not rocket-slogger logging.
    let mut config = Config::from(Config::figment());
    config.log_level = LogLevel::Off;

    rocket::custom(config)
        .attach(fairing)
        .mount(
            "/",
            routes![always_greet, always_thank, always_fail, dynamic_path],
        )
        .register("/", catchers![not_found])
}
