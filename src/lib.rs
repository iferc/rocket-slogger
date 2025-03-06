pub mod fairing;
pub mod from_request;

#[cfg(feature = "transactions")]
pub mod transaction;

// various slog re-exports for convenience
pub use slog::{o, o as log_fields, Drain, Logger};
// logging macros that are compiled away in release mode
pub use slog::{debug, trace};
// logging macros that are kept in all builds
pub use slog::{error, info, warn};

use rocket::{Request, Response};
use std::sync::Arc;

#[allow(unused_imports)]
use std::future::Future;
#[allow(unused_imports)]
use std::pin::Pin;

#[derive(Clone)]
pub struct Slogger {
    logger: Arc<Logger>,

    #[cfg(feature = "callbacks")]
    request_handlers: Vec<
        Arc<
            dyn for<'r> Fn(
                    Arc<Logger>,
                    &'r mut Request<'_>,
                )
                    -> Pin<Box<dyn Future<Output = Option<Arc<Logger>>> + Send + 'r>>
                + Send
                + Sync
                + 'static,
        >,
    >,

    #[cfg(feature = "callbacks")]
    response_handlers: Vec<
        Arc<
            dyn for<'r> Fn(
                    Arc<Logger>,
                    &'r Request<'_>,
                    &'r mut Response<'_>,
                )
                    -> Pin<Box<dyn Future<Output = Option<Arc<Logger>>> + Send + 'r>>
                + Send
                + Sync
                + 'static,
        >,
    >,
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
        use slog_envlogger::EnvLogger;
        use std::sync::Mutex;

        let bunyan_logger = slog_bunyan::with_name(name, std::io::stderr()).build();
        let env_logger = EnvLogger::new(bunyan_logger);
        let logger = Logger::root(Mutex::new(env_logger).fuse(), log_fields!());

        Self::from_logger(logger)
    }

    pub fn from_logger(logger: Logger) -> Self {
        Self {
            logger: Arc::new(logger),

            #[cfg(feature = "callbacks")]
            request_handlers: vec![],

            #[cfg(feature = "callbacks")]
            response_handlers: vec![],
        }
    }

    pub fn get(&self) -> &Logger {
        &self.logger
    }

    pub fn get_for_request(&self, request: &Request<'_>) -> Logger {
        let content_type = request.content_type().map(|format| format.to_string());
        let user_agent = request
            .headers()
            .get("user-agent")
            .collect::<Vec<_>>()
            .join("; ");

        #[cfg(not(feature = "transactions"))]
        let logger = self.logger.new(log_fields!(
            "user-agent" => user_agent,
            "content-type" => content_type,
        ));

        #[cfg(feature = "transactions")]
        let logger = {
            let transaction = transaction::RequestTransaction::new().attach_on(request);

            self.logger.new(log_fields!(
                "received" => transaction.received_as_string(),
                "transaction" => transaction.id_as_string(),

                "user-agent" => user_agent,
                "content-type" => content_type,
            ))
        };

        Self::new_logger_with_request_details(&logger, request)
    }

    pub fn get_for_response(&self, request: &Request<'_>, response: &Response<'_>) -> Logger {
        let content_type = response.content_type().map(|format| format.to_string());
        let status = response.status();

        #[cfg(not(feature = "transactions"))]
        let logger = self.logger.new(log_fields!(
            "content-type" => content_type,
            "reason" => status.reason().map(|reason| reason.to_string()),
            "code" => status.code,
        ));

        #[cfg(feature = "transactions")]
        let logger = {
            let transaction = transaction::RequestTransaction::new().attach_on(request);

            self.logger.new(log_fields!(
                "elapsed_ns" => transaction.elapsed_ns(),
                "received" => transaction.received_as_string(),
                "transaction" => transaction.id_as_string(),
                "content-type" => content_type,
                "reason" => status.reason().map(|reason| reason.to_string()),
                "code" => status.code,
            ))
        };

        Self::new_logger_with_request_details(&logger, request)
    }

    fn new_logger_with_request_details(logger: &Logger, request: &Request<'_>) -> Logger {
        if let Some(route) = request.route() {
            logger.new(log_fields!(
                "rank" => route.rank,
                "route" => route.name.as_ref().map(|route| route.to_string()),
                "path" => format!("{}", route.uri),
                "method" => format!("{}", route.method),
                "uri" => format!("{}", request.uri()),
            ))
        } else {
            logger.new(log_fields!(
                "method" => format!("{}", request.method()),
                "uri" => format!("{}", request.uri()),
            ))
        }
    }

    #[cfg(feature = "callbacks")]
    pub fn on_request(
        mut self,
        handler: impl for<'r> Fn(
                Arc<Logger>,
                &'r mut Request<'_>,
            )
                -> Pin<Box<dyn Future<Output = Option<Arc<Logger>>> + Send + 'r>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.request_handlers.push(Arc::new(handler));
        self
    }

    #[cfg(feature = "callbacks")]
    pub fn on_response(
        mut self,
        handler: impl for<'r> Fn(
                Arc<Logger>,
                &'r Request<'_>,
                &'r mut Response<'_>,
            )
                -> Pin<Box<dyn Future<Output = Option<Arc<Logger>>> + Send + 'r>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.response_handlers.push(Arc::new(handler));
        self
    }
}

impl From<Logger> for Slogger {
    fn from(logger: Logger) -> Self {
        Slogger::from_logger(logger)
    }
}

impl From<&Logger> for Slogger {
    fn from(logger: &Logger) -> Self {
        Slogger::from_logger(logger.clone())
    }
}

impl std::ops::Deref for Slogger {
    type Target = Logger;

    fn deref(&self) -> &Logger {
        &self.logger
    }
}
