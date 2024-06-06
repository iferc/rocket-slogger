use rocket_slogger::{info, log_fields, Slogger};

use rocket::{catch, get, post};

#[get("/")]
pub fn always_greet(log: Slogger) -> &'static str {
    info!(log, "Greeted");
    "Hello world"
}

#[post("/post")]
pub fn always_thank(log: Slogger) -> &'static str {
    info!(log, "Thanked");
    "Thank you"
}

#[get("/fail")]
pub fn always_fail(log: Slogger) -> &'static str {
    info!(log, "Uh oh...");
    todo!()
}

#[get("/<dynamic>")]
pub fn dynamic_path(log: Slogger, dynamic: String) -> &'static str {
    info!(log, "Received dynamic path"; log_fields!("dynamic" => dynamic));
    "Very interesting!"
}

#[catch(404)]
pub async fn not_found(req: &rocket::Request<'_>) -> String {
    let logger = req
        .guard::<Slogger>()
        .await
        // note that you probably shouldn't .unwrap() or .expect() production code
        .expect("Slogger should always be valid");

    // there are already logs for each user request so this is not a great use case
    info!(logger, "Confused by a user");

    format!("Could not find `{}`.", req.uri())
}
