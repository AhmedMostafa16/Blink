#[macro_use]
extern crate blink;

use futures::future::ok;

// Import traits and the reexported hyper and futures crates.
use blink::*;

fn main() {
    let server = Blink::new().routes(routes! {
    ("/test/:param/test2", Parameter)
    ("/test/:param/test2", PostParameter)
    ("/test/:user/test2", PostParameter)
    });
    server.run().unwrap();
}

routes!(
    (Get, Parameter, |req: Request| {
        println!("{:?}", req);
        ok(Response::new())
    })
    (Post, PostParameter, |req: Request| {
        println!("{:?}", req);
        ok(Response::new())
    })
);