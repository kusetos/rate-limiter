use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, http::StatusCode,
    web::Data,
}; 
use futures_util::future::{LocalBoxFuture, ready, Ready};
use std::{
    collections::HashMap,
    rc::Rc,
    sync::Mutex,
    task::{Context, Poll},
    time::Instant,
};

#[derive(Clone)]
pub struct RateLimiterConfig {
    pub capacity: f64,
    pub refill_rate: f64,
    pub client_id_header: String,
}

pub struct RateLimiter;

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware { service: Rc::new(service) }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        let config = req
            .app_data::<Data<RateLimiterConfig>>()
            .expect("RateLimiterConfig not found in app data")
            .clone();

        let buckets = req
            .app_data::<Data<Mutex<HashMap<String, (f64, Instant)>>>>()
            .unwrap()
            .clone();

        Box::pin(async move {
            // Identify client
            let client_id = if config.client_id_header == "ip" {
                req.connection_info()
                    .peer_addr()
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                req.headers()
                    .get(&config.client_id_header)
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string()
            };

            let mut map = buckets.lock().unwrap();
            let now = Instant::now();
            let (tokens, last) = map
                .entry(client_id)
                .or_insert((config.capacity, now));

            let elapsed = now.duration_since(*last).as_secs_f64();
            *tokens = (*tokens + elapsed * config.refill_rate).min(config.capacity);
            *last = now;

            if *tokens >= 1.0 {
                *tokens -= 1.0;
                drop(map);

                let res: ServiceResponse<B> = srv.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                let too_many = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
                    .body("Too many requests: rate limit exceeded")
                    .map_into_right_body();
                Ok(req.into_response(too_many))
            }
        })
    }
}
