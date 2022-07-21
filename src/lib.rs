//! # rustgram
//! A fast server framework for [hyper](https://github.com/hyperium/hyper)
//!
//! This framework provides a full automatic app builder from yml files
//!
//! ````ignore
//! use rustgram::{r, Router, Request, Response};
//! use std::net::SocketAddr;
//!
//! async fn not_found_handler(_req: Request) -> Response
//! {
//! 	return hyper::Response::builder()
//! 		.status(hyper::StatusCode::NOT_FOUND)
//! 		.body("Not found".into())
//! 		.unwrap();
//! }
//!
//! pub async fn test_handler(_req: Request) -> String
//! {
//! 	format!("test called")
//! }
//!
//! #[tokio::main]
//! async fn main()
//! {
//! 	let mut router = Router::new(not_found_handler);
//! 	router.get("/", r(test_handler));
//! 	router.get("/api", r(test_handler));
//!
//! 	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//!
//! 	//start the app
//! 	rustgram::start(router, addr).await;
//! }
//! ````

#[cfg(feature = "route_builder")]
mod builder;
mod router;
pub mod service;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
pub use router::route::{r, RouteParams};
pub use service::gram_error::{GramHttpErr, GramStdHttpErr};

#[cfg(feature = "route_builder")]
pub use self::builder::route_parser;
use crate::router::Router as CoreRouter;

/**
# Router

The core router with hyper request and response.

````ignore
use rustgram::{r, Router};

//Not found handler, which is called when no route matched the request
let mut router = Router::new(crate::not_found_handler);

//insert a get route
router.get("/", r(test_handler::test_handler));

//post
router.post("/", r(test_handler::test_handler));
````
*/
pub type Router = CoreRouter<Request, Response>;

/**
# hyper response with the hyper body
*/
pub type Response = hyper::Response<hyper::Body>;

/**
# hyper request with the hyper body
*/
pub type Request = hyper::Request<hyper::Body>;

/**
# Start listen for incoming connections

Build the hyper service and on connection, start the router matcher.

Build http 1.1 server.
*/
pub async fn start(router: Router, addr: SocketAddr)
{
	let shared_app = Arc::new(router);

	let new_service = make_service_fn(move |_| {
		//this function will call for every connection
		//init the app service
		let app_capture = shared_app.clone();

		async {
			//return the result as async block
			Ok::<_, Infallible>(service_fn(move |req| {
				//this function will call for every request
				handle_req(app_capture.clone(), req)
			}))
		}
	});

	let server = Server::bind(&addr).serve(new_service);

	println!("Listening on http://{}", addr);

	server.await.unwrap();
}

/**
# handle the req with Arc Router pointer

invoke the matched route.
The matched route is a ref to a Box pointer
*/
async fn handle_req(router: Arc<Router>, mut req: Request) -> Result<Response, Infallible>
{
	let found = router.handle_req(req.method(), req.uri().path());

	req.extensions_mut().insert(found.params);

	let res = found.handler.invoke(req).await;

	Ok(res)
}
