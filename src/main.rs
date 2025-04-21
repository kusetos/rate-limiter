use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use dotenv::dotenv;
use std::env;
use redis::aio::ConnectionManager;
use redis::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let client = Client::open(redis_url).expect("Cannot create Redis client");
    let manager = ConnectionManager::new(client)
        .await
        .expect("Cannot connect to Redis");

    let max_requests: u32 = env::var("MAX_REQUESTS").unwrap_or_else(|_| "100".to_string()).parse().unwrap();
    let window_secs: usize = env::var("WINDOW_SECS").unwrap_or_else(|_| "60".to_string()).parse().unwrap();
    let client_id_header = env::var("CLIENT_ID").unwrap_or_else(|_| "ip".to_string());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(manager.clone()))
            .app_data(web::Data::new(middleware::RateLimiterConfig {
                max_requests,
                window_secs,
                client_id_header: client_id_header.clone(),
            }))
            .wrap(middleware::RateLimiter)
            .route("/", web::get().to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}