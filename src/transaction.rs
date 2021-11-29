use chrono::{DateTime, Local};
use rocket::Request;
use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
pub struct RequestTransaction {
    pub id: Uuid,
    pub received: DateTime<Local>,
}

impl RequestTransaction {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            received: Local::now(),
        }
    }

    pub fn attach_on<'r>(self, request: &'r Request<'_>) -> &'r Self {
        request.local_cache(|| self)
    }

    pub fn id_as_string(&self) -> String {
        self.id
            .to_hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_string()
    }

    pub fn received_as_string(&self) -> String {
        self.received.to_rfc3339()
    }

    pub fn elapsed_as_string(&self) -> String {
        (Local::now() - self.received).to_string()
    }

    pub fn elapsed_ns(&self) -> Option<i64> {
        (Local::now() - self.received).num_nanoseconds()
    }
}
