extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate serde_json;
extern crate wal_rs;

use std::io::Cursor;

#[macro_use]
extern crate serde_derive;
extern crate json;

use actix_web::{
    error, http, middleware, server, App, AsyncResponder, Error, HttpMessage, HttpRequest,
    HttpResponse, Json,
};

use futures::{Future, Stream};

use bytes::Buf;
use wal_rs::*;

use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    name: String,
}

/// This handler uses `HttpRequest::json()` for loading json object.
fn new_task(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    req.json()
        .from_err() // convert all errors into `Error`
        .and_then(|val: Task| {
            println!("model: {:?}", val);

            let mut buf = Cursor::new(val.name.as_bytes());

            req.state()
                .wal
                .lock()
                .unwrap()
                .write(&[buf.get_u8()])
                .unwrap();

            Ok(HttpResponse::Ok().json(val)) // <- send response
        })
        .responder()
}

/// Application state
struct AppState {
    wal: Arc<Mutex<WAL>>,
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("server");

    let cfg = Config {
        entry_per_segment: 100,
        check_crc32: false,
    };

    let mut wal = WAL::open("./testdir", cfg).unwrap();

    let wal = Arc::new(Mutex::new(wal));

    server::new(move || {
        // App::with_state(app_state)

        App::with_state(AppState { wal: wal.clone() }) // <- create app with shared state
            // enable logger
            .middleware(middleware::Logger::default())
            .resource("/task", |r| r.method(http::Method::POST).f(new_task))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .shutdown_timeout(1)
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}

// #![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
// //! There are two level of statefulness in actix-web. Application has state
// //! that is shared across all handlers within same Application.
// //! And individual handler can have state.
// //!
// //! > **Note**: http server accepts an application factory rather than an
// //! application > instance. Http server constructs an application instance for
// //! each thread, > thus application state
// //! > must be constructed multiple times. If you want to share state between
// //! different > threads, a shared object should be used, e.g. `Arc`.
// //!
// //! Check [user guide](https://actix.rs/book/actix-web/sec-2-application.html) for more info.

// extern crate actix;
// extern crate actix_web;
// extern crate env_logger;
// extern crate futures;
// extern crate serde_json;

// #[macro_use]
// extern crate serde_derive;
// extern crate json;

// use std::sync::Arc;
// use std::sync::Mutex;

// use futures::{Future, Stream};

// use actix_web::{
//     http, middleware, server, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse,
//     Json,
// };

// /// Application state
// struct AppState {
//     counter: Arc<Mutex<usize>>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Task {
//     name: String,
// }

// /// simple handle
// fn index(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
//     req.json()
//         .from_err() // convert all errors into `Error`
//         .and_then(|val: Task| {
//             println!("model: {:?}", val);

//             // let mut buf = Cursor::new(val.name.as_bytes());

//             // req.state().wal.write(&[buf.get_u8()]).unwrap();

//             Ok(HttpResponse::Ok().json(val)) // <- send response
//         })
//         .responder()
// }

// fn main() {
//     ::std::env::set_var("RUST_LOG", "actix_web=info");
//     env_logger::init();
//     let sys = actix::System::new("ws-example");

//     let counter = Arc::new(Mutex::new(0));
//     //move is necessary to give closure below ownership of counter
//     server::new(move || {
//         App::with_state(AppState {
//             counter: counter.clone(),
//         }) // <- create app with shared state
//         // enable logger
//         .middleware(middleware::Logger::default())
//         // register simple handler, handle all methods
//         .resource("/", |r| r.f(index))
//     })
//     .bind("127.0.0.1:8080")
//     .unwrap()
//     .start();

//     println!("Started http server: 127.0.0.1:8080");
//     let _ = sys.run();
// }
