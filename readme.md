# Http2

![Lint & test & build](https://github.com/stockwayup/http2/actions/workflows/main.yml/badge.svg)

The second version of a lightweight microservice, rewritten from Go to Rust.
It acts as a web server to handle incoming HTTP requests and forward them for asynchronous processing by a backend service.
This version uses NATS for messaging, ensuring high performance, scalability, and reliability.
