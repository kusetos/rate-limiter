mod rate_limiter;

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use dotenv::dotenv;
use std::env;
use std::sync::Mutex;
use std::collections::HashMap;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let capacity: f64 = env::var("CAPACITY").unwrap_or_else(|_| "100".into()).parse().unwrap();
    let refill_rate: f64 = env::var("REFILL_RATE").unwrap_or_else(|_| "1".into()).parse().unwrap();
    let client_id_header = env::var("CLIENT_ID").unwrap_or_else(|_| "ip".into());

    
    let buckets = web::Data::new(Mutex::new(HashMap::<String, (f64, std::time::Instant)>::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(buckets.clone())
            .app_data(web::Data::new(rate_limiter::RateLimiterConfig {
                capacity,
                refill_rate,
                client_id_header: client_id_header.clone(),
            }))
            .wrap(rate_limiter::RateLimiter)
            .route("/", web::get().to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("More Points Pls")
}