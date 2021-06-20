mod iex_cloud;
mod utils;

extern crate log;

use crate::iex_cloud::QuoteResponse;
use actix_web::web::Query;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use dotenv::dotenv;
use futures::future::join_all;
use serde::Deserialize;
use std::sync::Mutex;
use std::{
    env,
    fmt::{self, Display},
};
use termion::color;
use utils::{comma_separated, is_plaintext_agent};
use validator::{Validate, ValidationError};

// TODO: extract as a formatter?
// TODO: formatting with emojis? 🚀💎🙌

impl Display for QuoteResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.change_percent > 0. {
            write!(
                f,
                "{} ${} {}",
                self.symbol,
                self.delayed_price,
                self.change()
            )
        } else {
            write!(
                f,
                "{} ${} {}",
                self.symbol,
                self.delayed_price,
                self.change()
            )
        }
    }
}

impl QuoteResponse {
    fn change(&self) -> String {
        if self.change_percent > 0. {
            return format!(
                "{}+{} (+{}){}",
                color::Fg(color::Green),
                self.change,
                self.change_percent * 100.,
                color::Fg(color::Reset)
            );
        } else if self.change_percent < 0. {
            return format!(
                "{}{} ({}){}",
                color::Fg(color::Red),
                self.change,
                self.change_percent * 100.,
                color::Fg(color::Reset)
            );
        }
        String::from("")
    }
}

#[get("/quote/{ticker}")]
async fn quote(
    data: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let ticker = path.into_inner();
    let client = data.iex_client.lock().unwrap();
    let v = client.get_quote(ticker).await;
    if is_plaintext_agent(req.headers().get("User-Agent").unwrap().to_str().unwrap()) {
        Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body(format!("{}", v)))
    } else {
        Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body(format!("currently we support plaintext agents only")))
    }
}

fn default_separator() -> String {
    " ".to_string()
}

fn default_precision() -> i8 {
    2
}

fn validate_tickers_query<T>(v: &Vec<T>) -> Result<(), ValidationError> {
    if v.len() < 1 || v.len() > 10 {
        return Err(ValidationError::new("invalid number of tickers"));
    }
    Ok(())
}

#[derive(Validate, Deserialize)]
struct QuotesQuery {
    #[serde(deserialize_with = "comma_separated")]
    #[validate(custom = "validate_tickers_query")]
    tickers: Vec<String>,

    #[serde(default = "default_separator")]
    separator: String,

    #[serde(default = "default_precision")]
    precision: i8,
}

#[get("/quote")]
async fn quotes(
    data: web::Data<AppState>,
    mut info: Query<QuotesQuery>,
) -> Result<HttpResponse, Error> {
    let client = data.iex_client.lock().unwrap();
    // TODO: limit mux number of requested tickesr
    // get all requested quotes asynchronously
    let results = join_all(
        info.tickers
            .iter_mut()
            .map(|ticker| client.get_quote(ticker.to_string())),
    )
    .await;
    let result = results
        .iter()
        .map(|q| format!("{}", q))
        .collect::<Vec<String>>()
        .join(&info.separator);
    Ok(HttpResponse::Ok().content_type("text/plain").body(result))
}

// TODO: endpoint to get multiple tickers at once?
// TODO: endpoint for crypto?
// TODO: endpoint for ETFs?
// TODO: endpoint for indexes??

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("🚀"))
}

struct AppState {
    iex_client: Mutex<iex_cloud::Client>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("starting up");
    dotenv().ok();

    env::set_var("RUST_LOG", "actix_todo=debug,actix_web=info");
    env_logger::init();

    let token = env::var("IEX_CLOUD_TOKEN").expect("IEX_CLOUD_TOKEN must be set");

    let app_data = web::Data::new(AppState {
        iex_client: Mutex::new(iex_cloud::Client::new(token)),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(quotes)
            .service(quote)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
