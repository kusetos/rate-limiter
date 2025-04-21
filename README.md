
This microservice implements an in-memory Token Bucket rate limiting algorithm
# Key Features

- Algorithm: Token Bucket
- Configurable: Bucket capacity, refill rate via environment variables
- Actix-web middleware: Custom "RateLimiter" [rate limiter mod](src/rate_limiter.rs), 
# How It Works

1. Each client has a **bucket** with a maximum number of tokens (`CAPACITY`).
2. Tokens are **refilled** at a constant rate (`REFILL_RATE` tokens per second).
3. On each request, the rate_limiter:
- Calculates the elapsed time since the last refill and adds tokens based on `REFILL_RATE`.
- If the bucket has more than 1 token, consumes one token and allows the request.
- Otherwise, returns "429 Too Many Requests".
![image](https://github.com/user-attachments/assets/387516cb-6b67-4262-bbfc-6a57d66e3720)
