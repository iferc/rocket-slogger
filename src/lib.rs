pub mod fairing;
pub mod from_request;

#[cfg(feature = "transaction")]
pub mod transaction;

// various slog re-exports for convenience
pub use slog::{o, o as log_fields, Drain, Logger};
// logging macros that are compiled away in release mode
pub use slog::{debug, trace};
// logging macros that are kept in all builds
pub use slog::{error, info, warn};

use rocket::{Request, Response};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Slogger {
    logger: Arc<Logger>,
}

impl Slogger {
    #[cfg(feature = "terminal")]
    pub fn new_terminal_logger() -> Self {
        use slog_term::{FullFormat, PlainSyncDecorator};

        let plain = PlainSyncDecorator::new(std::io::stdout());
        let logger = Logger::root(FullFormat::new(plain).build().fuse(), log_fields!());

        Self::from_logger(logger)
    }

    #[cfg(feature = "bunyan")]
    pub fn new_bunyan_logger(name: &'static str) -> Self {
        use std::sync::Mutex;

        let bunyan_logger = slog_bunyan::with_name(name, std::io::stderr()).build();
        let logger = Logger::root(Mutex::new(bunyan_logger).fuse(), log_fields!());

        Self::from_logger(logger)
    }

    pub fn from_logger(logger: Logger) -> Self {
        Self {
            logger: Arc::new(logger),
        }
    }

    pub fn get(&self) -> &Logger {
        &*self.logger
    }

    pub fn get_for_request(&self, request: &Request<'_>) -> Logger {
        let content_type = request.content_type().map(|format| format.to_string());
        let user_agent = request
            .headers()
            .get("user-agent")
            .collect::<Vec<_>>()
            .join("; ");

        #[cfg(not(feature = "transaction"))]
        let logger = self.logger.new(log_fields!(
            "user-agent" => user_agent,
            "content-type" => content_type,
        ));

        #[cfg(feature = "transaction")]
        let logger = {
            let transaction = transaction::RequestTransaction::new().attach_on(&request);

            self.logger.new(log_fields!(
                "received" => transaction.received_as_string(),
                "transaction" => transaction.id_as_string(),

                "user-agent" => user_agent,
                "content-type" => content_type,
            ))
        };

        Self::new_logger_with_request_details(&logger, request)
    }

    pub fn get_for_response(&self, request: &Request<'_>, _: &Response<'_>) -> Logger {
        let logger = &*self.logger;

        #[cfg(feature = "transaction")]
        let new_logger = {
            let transaction = transaction::RequestTransaction::new().attach_on(&request);

            logger.new(log_fields!(
                "elapsed_ns" => transaction.elapsed_ns(),
                "received" => transaction.received_as_string(),
                "transaction" => transaction.id_as_string(),
            ))
        };
        #[cfg(feature = "transaction")]
        let logger = &new_logger;

        Self::new_logger_with_request_details(logger, request)
    }

    fn new_logger_with_request_details(logger: &Logger, request: &Request<'_>) -> Logger {
        if let Some(route) = request.route() {
            logger.new(log_fields!(
                "rank" => route.rank,
                "route" => route.name.as_ref().map(|route| route.to_string()),
                "path" => format!("{}", route.uri),
                "method" => format!("{}", route.method),
            ))
        } else {
            logger.new(log_fields!(
                "path" => format!("{}", request.uri()),
                "method" => format!("{}", request.method()),
            ))
        }
    }
}

impl std::ops::Deref for Slogger {
    type Target = Logger;

    fn deref(&self) -> &Logger {
        &*self.logger
    }
}
