use crate::Slogger;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Slogger {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Slogger, ()> {
        match request.guard::<&State<Slogger>>().await {
            Outcome::Success(slogger) => {
                let logger = slogger.get_for_request(request);
                rocket::outcome::Outcome::Success(Slogger::from_logger(logger))
            }

            _ => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}
