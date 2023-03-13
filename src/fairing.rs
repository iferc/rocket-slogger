use crate::{info, Slogger};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Build, Config, Data, Orbit, Request, Response, Rocket};
use std::sync::Arc;

#[inline]
fn url_from_rocket_config(config: &Config) -> String {
    format!(
        "{scheme}://{address}:{port}",
        scheme = if config.tls_enabled() {
            "https"
        } else {
            "http"
        },
        address = &config.address,
        port = &config.port
    )
}

#[inline]
fn temp_dir_path_from_rocket_config(config: &Config) -> String {
    config
        .temp_dir
        .relative()
        .into_os_string()
        .into_string()
        .unwrap_or_else(|_| String::from(""))
}

#[rocket::async_trait]
impl Fairing for Slogger {
    fn info(&self) -> Info {
        Info {
            name: "Slog Fairing",
            kind: Kind::Ignite | Kind::Liftoff | Kind::Request | Kind::Response,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
        Ok(rocket.manage(self.clone()))
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let config = rocket.config();

        let url = url_from_rocket_config(config);
        let temp_dir_string = temp_dir_path_from_rocket_config(config);

        info!(
            &self.logger,
            "Rocket Launched";
            "log_level" => %config.log_level,
            "temp_dir" => temp_dir_string,
            "ident" => %config.ident,
            "tls" => config.tls_enabled(),
            "limits" => %config.limits,
            "keep_alive" => config.keep_alive,
            "workers" => config.workers,
            "port" => config.port,
            "host" => %config.address,
            "url" => %url,
            "profile" => %config.profile,
        );

        for route in rocket.routes() {
            info!(
                &self.logger,
                "Route Registered";
                "rank" => route.rank,
                "route" => route.name.as_ref().map(|route| route.to_string()),
                "content-type" => route.format.as_ref().map(|format| format.to_string()),
                "path" => %route.uri,
                "url" => format!("{}{}", url, route.uri),
                "method" => %route.method,
            );
        }

        for catcher in rocket.catchers() {
            info!(
                &self.logger,
                "Catcher Registered";
                "route" => catcher.name.as_ref().map(|catcher| catcher.to_string()),
                "code" => catcher.code,
                "path" => %catcher.base,
                "url" => format!("{}{}", url, catcher.base),
            );
        }

        info!(
            &self.logger,
            "Accepting Connections";
            "port" => config.port,
            "host" => %config.address,
            "url" => url,
        );
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        #[allow(unused_mut)]
        let mut logger = Arc::new(self.get_for_request(request));

        #[cfg(feature = "callbacks")]
        for handler in &self.request_handlers {
            if let Some(new_logger) = handler(logger.clone(), request).await {
                logger = new_logger;
            }
        }

        info!(logger, "Request");
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        #[allow(unused_mut)]
        let mut logger = Arc::new(self.get_for_response(request, response));

        #[cfg(feature = "callbacks")]
        for handler in &self.response_handlers {
            if let Some(new_logger) = handler(logger.clone(), request, response).await {
                logger = new_logger;
            }
        }

        let status = response.status();
        let body_size = response.body_mut().size().await;

        info!(
            logger,
            "Response";
            "reason" => status.reason().map(|reason| reason.to_string()),
            "code" => status.code,
            "size" => body_size,
        );
    }
}
