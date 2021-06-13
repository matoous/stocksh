mod iex_cloud;

extern crate log;

use actix_web::{get, web, App, HttpServer, HttpResponse, middleware, Error, HttpRequest};
use crate::iex_cloud::QuoteResponse;
use std::sync::Mutex;
use std::fmt::Display;
use std::{env, fmt};
use termion::color;
use dotenv::dotenv;

// TODO: extract as a formatter?
// TODO: formatting with emojis? 🚀💎🙌

impl Display for QuoteResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.change_percent > 0. {
            write!(f, "{} ${} {}", self.symbol, self.delayed_price, self.change())
        } else {
            write!(f, "{} ${} {}", self.symbol, self.delayed_price, self.change())
        }
    }
}

impl QuoteResponse {
    fn change(&self) -> String {
        if self.change_percent > 0. {
            return format!("{}+{} (+{}){}", color::Fg(color::Green), self.change, self.change_percent * 100., color::Fg(color::Reset));
        } else if self.change_percent < 0. {
            return format!("{}{} ({}){}", color::Fg(color::Red), self.change, self.change_percent * 100., color::Fg(color::Reset));
        }
        String::from("")
    }
}

static PLAIN_TEXT_AGENTS: &'static [&str] = &[
    "curl",
    "httpie",
    "lwp-request",
    "wget",
    "python-requests",
    "openbsd ftp",
    "powershell",
    "fetch",
    "aiohttp",
];

fn is_plaintext_agent(agent: &str) -> bool {
    return PLAIN_TEXT_AGENTS.iter().any(
        |s| agent.to_lowercase().contains(s)
    );
}

#[get("/quote/{ticker}")]
async fn index(data: web::Data<AppState>, req: HttpRequest, path: web::Path<String>) -> Result<HttpResponse, Error> {
    let ticker = path.into_inner();
    let client = data.iex_client.lock().unwrap();
    let v = client.get_quote(ticker).await;
    if is_plaintext_agent(req.headers().get("User-Agent").unwrap().to_str().unwrap()) {
        Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body(format!("{}", v)))
    } else {
        Ok(HttpResponse::Ok().content_type("text/plain").body(format!("HI!")))
    }
}

// TODO: endpoint to get multiple tickers at once?

struct AppState {
    iex_client: Mutex<iex_cloud::Client>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env::set_var("RUST_LOG", "actix_todo=debug,actix_web=info");
    env_logger::init();

    let token = env::var("IEX_CLOUD_TOKEN").expect("IEX_CLOUD_TOKEN must be set");

    let app_data = web::Data::new(AppState { iex_client: Mutex::new(iex_cloud::Client::new(token)) });

    HttpServer::new(move || App::new()
        .app_data(app_data.clone())
        .wrap(middleware::Logger::default())
        .service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
