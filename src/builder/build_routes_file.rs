use crate::builder::entities::Route;

pub(crate) fn build_routes(routes: Vec<Route>, std_service: String, std_mw: String, handler_404: Option<String>) -> String
{
	let route_404 = match handler_404 {
		Some(h) => h,
		None => {
			//TODO system 404 handler
			//route_404 = format!("Response::new(hyper::Body::from({})), ", "\"404\"");
			panic!("no 404 handler");
		},
	};

	let router_sting = format!("let mut router = Router::new({}); ", route_404);

	//build the routes: router.get("/", r(handler));

	let mut routes_string = "".to_string();

	for route in routes {
		//route method is the same as the router method fn
		routes_string += &*format!("router.{}(\"{}\",{}); ", route.method, route.path, route.route);
	}

	format!(
		r"/**
# Generated route files by rustgram route builder.

Please do not modify this file. Any changes will be overridden by the next route build.
Use the returned router instead
 */
use rustgram::{{r, Router}};
use crate::{}::*;
use crate::{}::*;

pub(crate) fn routes() -> Router
{{
	{}
	{}

	router
}}
	",
		std_service, std_mw, router_sting, routes_string
	)
}
