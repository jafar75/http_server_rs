# http_server_rs

Rust implementation of the C10K problem — a high-performance HTTP/1.1 server using asynchronous I/O and event multiplexing.

---

## Overview

`http_server_rs` is a simple **HTTP/1.1 server** written in Rust, designed to efficiently handle thousands of concurrent connections using [**epoll**](https://man7.org/linux/man-pages/man7/epoll.7.html) as an advanced and very efficient I/O multiplexing feature offered by Linux kernel. It demonstrates a clean architecture for building low-latency network applications.

---

## Features

- Handles multiple clients concurrently using **epoll**.
- Supports **HTTP/1.1 GET requests**.
- Serves static files.
- Minimal, zero-dependency design for performance and simplicity.

---

## Supported Routes

Currently, the server has two basic routes:

1. `/hello.html` – returns a static HTML file.
2. `/` – returns a default message like `"Welcome to http_server_rs"`.


---

## How It Works

- The server uses **epoll** to efficiently monitor multiple sockets and handle I/O events without blocking.
- Each new connection is accepted and added to the epoll instance.
- When a socket is ready to read or write, the server processes the request and sends the response.
- Designed to scale to thousands of concurrent connections (C10K problem).

---

## TODO / Future Improvements

- Add **io_uring** support to compare performance against epoll.
- Implement **configurable logging** with different verbosity levels.
- Support **persistent connections (keep-alive)** and pipelining.
- Add **dynamic routing** for multiple endpoints.
- Support **HTTP/1.1 POST requests** and request body parsing.
- Add **metrics and monitoring** (e.g., requests/sec, latency).

---

## Usage

```bash
cargo run --release
```
Then visit:
- [http://localhost:8080/](http://localhost:8080/) – default message  
- [http://localhost:8080/hello.html](http://localhost:8080/hello.html) – static HTML file
