# Rustgram
##### A lightweight, fast and easy to use http routing and middleware framework build on top of [hyper](https://github.com/hyperium/hyper)

### Features
- build routes and middleware like Tower services
- uses yaml files to define routes

### Install in `Cargo.toml`
````toml
[dependencies]
# hyper when using Statuscode
hyper = { version = "0.14", features = ["full"] }

# tokio for the async main fn
tokio = { version = "1", features = ["full"] }

# rustgram
rustgram = "0.1"
````

## Documentation

### Getting started

1. Create the router with a not found handler service (e.g. a function)
2. adding routes, for every http method there is a router function: get, post, put, delete, head, ...
    1. use the r function to pass the handler to the router
    2. use the method function to define on what method this route should be matched to the given path
    3. set an url path
3. enter the socket address to listen for connection

````rust
use hyper::StatusCode;
use rustgram::{r, Router, Request,Response};
use std::net::SocketAddr;

async fn not_found_handler(_req: Request) -> Response
{ 
    return hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not found".into())
        .unwrap();
}

pub async fn test_handler(_req: Request) -> String
{ 
    format!("test called")
}

#[tokio::main]
async fn main()
{ 
    let mut router = Router::new(crate::not_found_handler);
    router.get("/", r(test_handler));
    router.get("/api", r(test_handler));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    //start the app
    rustgram::start(router, addr).await;
}
````

### Middleware

- A middleware is a service.
- The middleware calls the next service (which is obtained by the middleware transformation).
- A transform is used to return a new middleware with a next service
- The transform function is called everytime when a middleware is applied to a route

The middleware stack is build like a service call stack.

The Order of the middleware stack is reverse to the applied order.

````rust
use hyper::StatusCode;
use rustgram::service::{Service, ServiceTransform};
use rustgram::{Request, Response};

//define a middleware service
pub struct Middleware<S>
{ 
    inner: S,   //space the inner service to call it later
}

impl<S> Service<Request> for Middleware<S> 
where 
    S: Service<Request, Output = Response>, //define the return types from the next service
{
	type Output = S::Output;
	type Future = S::Future;
    
    fn call(&self, req: Request) -> Self::Future 
    {
		// before the request handler from the router is called
        self.inner.call(req)  //call the next handler 
		// after the request handler is called with the response 
    }
}

//define a middleware transform
pub struct MiddlewareTransform;

impl<S> ServiceTransform<S> for MiddlewareTransform 
where 
    S: Service<Request, Output = Response>, //define the return types from the next service
{ 
    type Service = Middleware<S>;
    
    fn transform(&self, inner: S) -> Self::Service 
    { 
        Middleware { 
            inner, 
        } 
    }
}

//or define a middleware transform with a function
pub fn middleware_transform<S>(inner: S) -> Middleware<S>
{ 
    Middleware { 
        inner, 
    }
}

async fn not_found_handler(_req: Request) -> Response
{ 
    return hyper::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("Not found".into())
        .unwrap();
}

pub async fn test_handler(_req: Request) -> String
{ 
    format!("test called")
}

//Apply a middleware to a route after the r function

#[tokio::main]
async fn main()
{ 
    let mut router = Router::new(crate::not_found_handler);
    router.get("/", r(test_handler)
        .add(middleware_transform)
    );
    
    router.get("/api", r(test_handler)
        .add(MiddlewareTransform)
    );
    
    //apply multiple middleware to a route
    router.get("/multiple", r(test_handler)
        .add(MiddlewareTransform)   //then this at last
        .add(middleware_transform)  //then this ^
        .add(middleware_transform)  //then this ^
        .add(middleware_transform)  //then this ^
        .add(middleware_transform)  //then this ^
        .add(middleware_transform)  //this is called first ^
    );
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    
    //start the app
    rustgram::start(router, addr).await;
}
````

#### Middleware with async handling

If work must be done before the req handling or after the response, a Box Future is needed

When calling async actions before the response, here Arc is needed to avoid lifetime issues:

- use a Pin Box feature
- use inner service as an arc pointer
- clone the arc pointer before calling the async block
- do the async action in the async block

````rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rustgram::service::{Service, ServiceTransform};
use rustgram::{Request, Response};

pub struct Middleware<S>
{
	inner: Arc<S>,   //use Arc here to avoid lifetime issues
}

impl<S> Service<Request> for Middleware<S> 
where 
    S: Service<Request, Output = Response>,
{ 
    type Output = S::Output;
    type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;
    
    fn call(&self, req: Request) -> Self::Future 
    { 
        //must clone the service to call it in the async move block. (we are cloning the arc ref, to avoid lifetime issues)
        let next = self.inner.clone();
        
        Box::pin(async move { 
            //do async fn before req
            
            next.call(req).await 
            //do async fn after req 
        }) 
    }
}

pub fn mw_transform<S>(inner: S) -> Middleware<S>
{
	Middleware { 
        inner: Arc::new(inner), //use Arc here!
    }
}
````

Only after response async action:

````rust
use std::future::Future;
use std::pin::Pin;

use rustgram::service::{Service, ServiceTransform};
use rustgram::{Request, Response};

pub struct Middleware<S>
{
	inner: S,
}

impl<S> Service<Request> for Middleware<S>
	where
		S: Service<Request, Output = Response>,
{ 
    type Output = S::Output;
    type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;
    
    fn call(&self, req: Request) -> Self::Future 
    { 
        //no Arc cloning needed because we move the feature into the async block
        //but change the req is not possible here
        let res = self.inner.call(req);
        
        Box::pin(async move { 
            let res = res.await; 
            //do async fn after req 
            
            res
        }) 
    }
}

pub fn mw_transform<S>(inner: S) -> Middleware<S>
{
	Middleware { 
        inner, //no Arc here!
    }
}
````

### Handler return and error handling
The router only uses Service traits. For normal functions and closure, this is already implemented.

The functions don't need to return a Hyper Response, but their return gets converted into hyper response.

Supported returns are:
- Hyper Response
- String
- &'static str
- Result<String, GramStdHttpErr>
- Result<String, E>
- Result<R, E>

The GramStdHttpErr gets converted into a hyper response.

The E where E impl GramHttpErr, will convert via the GramHttpErr trait.

````rust
use hyper::StatusCode;
use rustgram::{GramHttpErr, Response, Request};

pub struct HttpErr
{ 
    http_status_code: u16, 
    api_error_code: u32, 
    msg: &'static str,
}

impl HttpErr
{ 
    pub fn new(http_status_code: u16, api_error_code: u32, msg: &'static str) -> Self 
    { 
        Self { 
            http_status_code, 
            api_error_code, 
            msg 
        } 
    }
}

impl GramHttpErr<Response> for HttpErr
{ 
    fn get_res(&self) -> Response 
    { 
        let status = match StatusCode::from_u16(self.http_status_code) { 
            Ok(s) => s, 
            Err(_e) => StatusCode::BAD_REQUEST, 
        };
        
        //the msg for the end user
        let msg = format!(
            "{{\"status\": {}, \"error_message\": \"{}\"}}", 
            self.api_error_code, self.msg
        );
        
        hyper::Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(hyper::Body::from(msg))
            .unwrap()
	}
}

//example usage:

pub async fn test_handler_err(_req: Request) -> Result<String, HttpErr>
{ 
    Err(HttpErr::new(400,1,"Input not valid"))
}
````

#### Result<R, E>

R can be any type which implements HttpResult.

Example to return a json string:

````rust
use rustgram::service::HttpResult;
use rustgram::{GramHttpErr, Response, Request};
use serde::Serialize;
use serde_json::to_string;

pub struct JsonResult<T: ?Sized + Serialize>(pub T);

impl<T: ?Sized + Serialize> HttpResult<Response> for JsonResult<T>
{ 
    fn get_res(&self) -> Response 
    { 
        //to string from serde_json
        let string = match to_string(&self.0) { 
            Ok(s) => s, 
            //the Http err from the previous example
            Err(_e) => return HttpErr::new(422,1,"Json to string error",None).get_res(), 
        };
        
        hyper::Response::builder()
            .header("Content-Type", "application/json")
            .body(string.into())
            .unwrap() 
    }
}

//in another file
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResultMsg(pub String);

pub async fn test_handler_json_result(_req: Request) -> Result<JsonResult<ResultMsg>, HttpErr>
{
	Ok(JsonResult(ResultMsg(String::from("Hello world"))))
}
````

### Route builder and groups

- groups can only be build by the route builder
- the builder parses a yml file and create a new route file. this file contains a function which returns a router (to use it later).
- all routes in a route shares the same middleware and the same prefix
- nested groups are also possible

1. Create a 2nd bin crate for the route builder.
2. This crate calls the builder function
3. Set the input and the output path (both relative to the current working directory)
4. install rustgram with the route_builder feature
5. Build and execute the route builder everytime the routes are changed
6. use the route function, from the new file, to get the router

````shell
# in a workspace just create a new crate
cargo new route_builder
````

In the Cargo-toml file:

````toml
rustgram = { version = "0.1", features = ["route_builder"] }
````

Open the main function in `src/main.rs`

````rust
use rustgram::route_parser;

fn main()
{ 
//input path: from the root of the current working directory 
// output path: into the app crate (created via cargo new app)
    route_parser::start(
        "routes.yml".to_string(), 
        "app/src/routes.rs".to_string()
    );
}
````

Create the route file.

````yaml
# define the namespace where the route handlers live
# or leave it emtpy and use the full path to the handler
base_handler: test_handler
# define the namespace for the middleware
base_mw: test_mw
# the 404 handler. is called when no routes matched the given path
handler_404: crate::not_found_handler
# prefix for all routes
prefix: "/"

# the routes and groups. use the method followed by the path (p) and the handler (s)
routes:
  - get:
      p: ""
      # must match the base_handler namespace
      s: test_handler::test_handler
  # a put route with middleware 
  - put:
      p: ""
      s: test_handler::test_handler
      mw:
        - mw1_transform
        - mw_transform
  # a group of routes.
  # a prefix (p) for all routes and middleware (mw) like routes
  - group:
      p: admin
      mw:
        - mw1_transform
        - mw_transform
      # the routes to this group
      gr:
        - get:
            p: ""
            s: test_handler_db::test_handler_db_to_json
        - get:
            p: "/user/:id"
            s: test_handler::test_handler
        # for this route, mw is called first, then mw1, mw2 and mw3
        - put:
            p: "/many_mw"
            s: test_handler::test_handler
            mw:
              - mw3_transform
              - mw2_transform
  - group:
      p: nested
      mw:
        - mw1_transform
        - mw_transform
      gr:
        # define a new group inside the group routes
        - group:
            p: "/management"
            mw:
              - mw1_transform
              - mw_transform
            gr:
              - put:
                  p: "/put"
                  s: test_handler::test_handler
                  mw:
                    - mw5_transform
                    - mw4_transform
````

This file is parsed to this:

````rust
/**
# Generated route files by rustgram route builder.

Please do not modify this file. Any changes will be overridden by the next route build.
Use the returned router instead
 */
use rustgram::{r, Router};

use crate::test_handler::*;
use crate::test_mw::*;

pub(crate) fn routes() -> Router
{ 
    let mut router = Router::new(crate::not_found_handler);
    router.get("/", r(test_handler::test_handler));
    
    router.put(
        "/", 
        r(test_handler::test_handler)
            .add(mw1_transform)
            .add(mw_transform), 
    );
    
    router.get(
        "/admin", 
        r(test_handler_db::test_handler_db_to_json)
            .add(mw1_transform)
            .add(mw_transform), 
    );
    
    router.get(
        "/admin/user/:id", 
        r(test_handler::test_handler)
            .add(mw1_transform)
            .add(mw_transform), 
    );
    
    router.put(
        "/admin/many_mw", 
        r(test_handler::test_handler)
            .add(mw3_transform)
            .add(mw2_transform)
            .add(mw1_transform)
            .add(mw_transform), 
    );
    
    router.put(
        "/nested/management/put", 
        r(test_handler::test_handler)
            .add(mw5_transform)
            .add(mw4_transform)
            .add(mw3_transform)
            .add(mw2_transform)
            .add(mw1_transform)
            .add(mw_transform), 
    );
    
    router
}
````