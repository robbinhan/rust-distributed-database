// extern crate bodyparser;
// extern crate iron;
// extern crate router;
// extern crate time;
// // extern crate wal_rs;

// use iron::prelude::*;
// use iron::status;
// use iron::{typemap, AfterMiddleware, BeforeMiddleware};
// use router::Router;
// use time::precise_time_ns;
// // use wal_rs::*;

// struct ResponseTime;

// impl typemap::Key for ResponseTime {
//     type Value = u64;
// }

// impl BeforeMiddleware for ResponseTime {
//     fn before(&self, req: &mut Request) -> IronResult<()> {
//         req.extensions.insert::<ResponseTime>(precise_time_ns());
//         Ok(())
//     }
// }

// impl AfterMiddleware for ResponseTime {
//     fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
//         let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
//         println!("Request took: {} ms", (delta as f64) / 1000000.0);
//         Ok(res)
//     }
// }

// fn hello_world(req: &mut Request) -> IronResult<Response> {
//     let ref query = req
//         .extensions
//         .get::<Router>()
//         .unwrap()
//         .find("msg")
//         .unwrap_or("/");

//     // let body = req.get::<bodyparser::Raw>();
//     // match body {
//     //     Ok(Some(body)) => println!("Read body:\n{}", body),
//     //     Ok(None) => println!("No body"),
//     //     Err(err) => println!("Error: {:?}", err),
//     // }

//     // let json_body = req.get::<bodyparser::Json>();
//     // match json_body {
//     //     Ok(Some(json_body)) => println!("Parsed body:\n{:?}", json_body),
//     //     Ok(None) => println!("No body"),
//     //     Err(err) => println!("Error: {:?}", err),
//     // }

//     // let struct_body = req.get::<bodyparser::Struct<MyStructure>>();
//     // match struct_body {
//     //     Ok(Some(struct_body)) => println!("Parsed body:\n{:?}", struct_body),
//     //     Ok(None) => println!("No body"),
//     //     Err(err) => println!("Error: {:?}", err),
//     // }

//     Ok(Response::with((status::Ok, *query)))
// }

// // const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;

// fn main() {
//     // let cfg = Config {
//     //     entry_per_segment: 100,
//     //     check_crc32: false,
//     // };

//     // let mut wal = WAL::open("./testdir", cfg).unwrap();

//     let mut router = Router::new(); // Alternative syntax:

//     let mut chain = Chain::new(hello_world);
//     chain.link_before(ResponseTime);
//     // chain.link_before(Read::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
//     chain.link_after(ResponseTime);

//     router.post("/task", chain, "index"); // let router = router!(index: get "/" => handler,

//     Iron::new(router).http("localhost:3000").unwrap();
// }

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

            // let mut buf = Cursor::new(val.name.as_bytes());

            // req.state().wal.write(&[buf.get_u8()]).unwrap();

            Ok(HttpResponse::Ok().json(val)) // <- send response
        })
        .responder()
}

/// Application state
struct AppState {
    wal: WAL,
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("json-example");

    let cfg = Config {
        entry_per_segment: 100,
        check_crc32: false,
    };

    let mut wal = WAL::open("./testdir", cfg).unwrap();

    server::new(|| {
        App::with_state(AppState { wal })
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
