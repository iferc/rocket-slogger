mod routes;

use rocket::config::Config;
use rocket::log::LogLevel;
use rocket::{catchers, routes, Build, Rocket};
use rocket_slogger::Slogger;
use routes::{always_fail, always_greet, always_thank, not_found};

#[allow(unused_imports)]
use std::future::Future;
#[allow(unused_imports)]
use std::pin::Pin;
#[allow(unused_imports)]
use std::sync::Arc;

#[cfg(all(feature = "bunyan", feature = "callbacks"))]
fn logger() -> Slogger {
    Slogger::new_bunyan_logger("My App")
        // response callback by function name
        .on_request(request_logger_callback)
        // callback as a closure function
        .on_request(|logger, _request| {
            // currently requires a pinned box to have an async context
            Box::pin(async move {
                // here any async function calls or server state can be fetched
                // so that it can be added to the logger that will form the response log
                let new_logger = logger.new(rocket_slogger::log_fields!(
                    "field:from-closure" => "some dynamic data derived at request time",
                    "in:request" => "more dynamic metrics",
                ));

                // the new logger must be returned in an Option<Arc<Logger>>
                Some(Arc::new(new_logger))
            })
        })
        // response callback by function name
        .on_response(response_logger_callback)
        // callback as a closure function
        .on_response(|logger, _request, _response| {
            // currently requires a pinned box to have an async context
            Box::pin(async move {
                // here any async function calls or server state can be fetched
                // so that it can be added to the logger that will form the response log
                let new_logger = logger.new(rocket_slogger::log_fields!(
                    "field:from-closure" => "some dynamic data derived at response time",
                    "in:response" => "more dynamic metrics",
                ));

                // the new logger must be returned in an Option<Arc<Logger>>
                Some(Arc::new(new_logger))
            })
        })
}

#[cfg(not(all(feature = "bunyan", feature = "callbacks")))]
fn logger() -> Slogger {
    todo!("Re-run this example with `--features bunyan,callbacks`")
}

#[cfg(feature = "callbacks")]
fn request_logger_callback<'r>(
    logger: Arc<rocket_slogger::Logger>,
    _request: &'r mut rocket::Request<'_>,
) -> Pin<Box<(dyn Future<Output = Option<Arc<rocket_slogger::Logger>>> + Send + 'r)>> {
    // if you import FutureExt from the rocket or futures crate,
    // then you can avoid wrapping the async block in `Box::pin` while instead calling .boxed() on it
    use rocket::futures::FutureExt;

    async move {
        // here any async function calls or server state can be fetched
        // so that it can be added to the logger that will form the response log
        let new_logger = logger.new(rocket_slogger::log_fields!(
            "field:from-closure" => "some dynamic data derived at request time",
            "in:request" => "more dynamic metrics",
        ));

        // the new logger must be returned in an Option<Arc<_>>
        Some(Arc::new(new_logger))
    }
    // currently requires a pinned box to have an async context
    .boxed()
}

#[cfg(feature = "callbacks")]
fn response_logger_callback<'r>(
    logger: Arc<rocket_slogger::Logger>,
    _request: &'r rocket::Request<'_>,
    _response: &'r mut rocket::Response<'_>,
    // if you import the BoxFuture alias from the rocket or futures crate,
    // then you can simplify the returning type as they are functionally the same
) -> rocket::futures::future::BoxFuture<'r, Option<Arc<rocket_slogger::Logger>>> {
    // currently requires a pinned box to have an async context
    Box::pin(async move {
        // here any async function calls or server state can be fetched
        // so that it can be added to the logger that will form the response log
        let new_logger = logger.new(rocket_slogger::log_fields!(
            "field:from-function" => "some dynamic data derived at response time",
            "in:response" => "more dynamic metrics",
        ));

        // the new logger must be returned in an Option<Arc<_>>
        Some(Arc::new(new_logger))
    })
}

#[rocket::launch]
async fn rocket() -> Rocket<Build> {
    // fairing built in another function just to ensure
    // that this example runs with the feature enabled
    let fairing = logger();

    let mut config = Config::from(Config::figment());
    config.log_level = LogLevel::Off;

    rocket::custom(config)
        .attach(fairing)
        .mount("/", routes![always_greet, always_thank, always_fail])
        .register("/", catchers![not_found])
}
