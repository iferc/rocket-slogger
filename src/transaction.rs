use chrono::DateTime;
use rocket::Request;
use uuid::Uuid;

#[cfg(feature = "local_time")]
type TimeZone = chrono::Local;

#[cfg(not(feature = "local_time"))]
type TimeZone = chrono::Utc;

#[derive(Copy, Clone, Debug)]
pub struct RequestTransaction {
    pub id: Uuid,
    pub received: DateTime<TimeZone>,
}

impl Default for RequestTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestTransaction {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            received: TimeZone::now(),
        }
    }

    pub fn attach_on<'r>(self, request: &'r Request<'_>) -> &'r Self {
        request.local_cache(|| self)
    }

    pub fn id_as_string(&self) -> String {
        self.id
            .hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_string()
    }

    pub fn received_as_string(&self) -> String {
        self.received.to_rfc3339()
    }

    pub fn elapsed_as_string(&self) -> String {
        (TimeZone::now() - self.received).to_string()
    }

    pub fn elapsed_ns(&self) -> Option<i64> {
        (TimeZone::now() - self.received).num_nanoseconds()
    }
}
