mod routes;

use rocket::config::Config;
use rocket::log::LogLevel;
use rocket::{catchers, routes, Build, Rocket};
use rocket_slogger::Slogger;
use routes::{always_fail, always_greet, always_thank, dynamic_path, not_found};

#[cfg(feature = "bunyan")]
fn logger() -> Slogger {
    Slogger::new_bunyan_logger("My App")
}

#[cfg(not(feature = "bunyan"))]
fn logger() -> Slogger {
    todo!("Re-run this example with `--features bunyan`")
}

#[rocket::launch]
async fn rocket() -> Rocket<Build> {
    // fairing built in another function just to ensure
    // that this example runs with the feature enabled
    let fairing = logger();

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
