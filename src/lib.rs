pub extern crate hyper;
pub extern crate futures;
extern crate url;

use std::net::SocketAddr;
use std::collections::HashMap;
use hyper::server::{Http, Service};
use hyper::{Request, Response, Method, StatusCode};
use futures::future::{FutureResult, ok, err};
use url::Url;
use std::io::{Error, ErrorKind};

macro_rules! ftry {
    ($exp: expr) => {
        match $exp {
            Ok(exp) => exp,
            Err(e) => return err(Io(Error::new(ErrorKind::Other, e))),
        }
    }
}

macro_rules! ftry_opt {
    ($exp: expr) => {
        match $exp {
            Some(exp) => exp,
            None => continue,
        }
    }
}

pub struct Blink {
    routes: HashMap<(Method, String), Box<Route<Method=Method>>>,
    port: u64,
}

impl Blink {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            port: 6767,
        }
    }

    pub fn routes(mut self, routes: HashMap<(Method, String), Box<Route<Method=Method>>>) -> Self {
        self.routes = routes;
        self
    }

    pub fn port(mut self, port_num: u64) -> Self {
        self.port = port_num;
        self
    }

    pub fn run(self) -> Result<(), hyper::Error>
    {
        let address_str = "127.0.0.1:".to_string() + &self.port.to_string();
        let address: SocketAddr = address_str.parse().unwrap();
        println!("Running server on {}", address);
        Http::new()
            .bind(&address, move || Ok(&self))?
            .run()?;
        Ok(())
    }
}

impl<'b> Service for &'b Blink {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let path = req.path().to_owned();
        let method = req.method().to_owned();
        match self.routes.get(&(method, path)) {
            Some(route) => {
                route.handler(req)
            }
            None => {
                // We couldn't find the url in our list of routes. This means that either:
                // - It has url parameters
                // - It doesn't match at all
                // We first check if it's a possible url parameter match and if it is call the
                // correct hanlder, otherwise we return a 404.
                use hyper::Error::Io;
                let base = ftry!(Url::parse("https://localhost"));
                for &(ref method, ref url) in self.routes.keys() {
                    // The methods don't match so skip them
                    if method != req.method() {
                        continue;
                    }
                    let url_in = ftry!(base.clone().join(&url));
                    let test = ftry!(base.clone().join(&req.path()));
                    let mut url_in = ftry_opt!(url_in.path_segments());
                    let test = ftry_opt!(test.path_segments());

                    let size1 = url_in.clone().count();
                    let size2 = test.clone().count();

                    if size1 != size2 {
                        continue;
                    }

                    let mut matched = true;
                    for (i, j) in url_in.zip(test) {
                        if i.starts_with(':') {
                            continue;
                        }

                        if i != j {
                            matched = false;
                            break;
                        }
                    }

                    if matched {
                        return self.routes.get(&(method.to_owned(), url.to_owned())).unwrap().handler(req);
                    }
                }
                ok(Response::new().with_status(StatusCode::NotFound))
            }
        }
    }
}

pub trait Route {
    type Method;
    fn handler(&self, _: Request) -> FutureResult<Response, hyper::Error>;
    fn method(&self) -> Self::Method;
}

#[macro_export]
macro_rules! router {
    ($( ($route: expr, $handler: expr))*) => {
        {
            use std::collections::HashMap;
            let mut map = HashMap::new();
            $(
                let boxed: Box<Route<Method = hyper::Method>> = Box::new($handler);
                map.insert((boxed.method(), $route.to_string()), boxed);
            )*
            map
        }
    };
}

#[macro_export]
macro_rules! routes {
    ($(($method: ident, $type: ident, $function: expr))*) => {
        use hyper::server::{ Request, Response };
        use futures::future::FutureResult;
        $(
        struct $type;
        impl Route for $type {
            type Method = hyper::Method;
            fn handler(&self, _req: Request) -> FutureResult<Response, hyper::Error> {
                #[inline(always)]
                $function(_req)
            }
            #[inline(always)]
            fn method(&self) -> Self::Method {
                hyper::Method::$method
            }
        }
        )*
    }
}
