use std::future::Future;
use std::pin::Pin;

use hyper::Method;

use crate::router::route::{GramRoute, Route};
use crate::service::Service;
use crate::RouteParams;

pub mod route;

type RouteVec<Req, Res> = Vec<Box<dyn Route<Req, Response = Res, Future = BoxedFut<Res>>>>;

type RouteId = usize;

type BoxedFut<Res> = Pin<Box<dyn Future<Output = Res> + Send>>;

pub(crate) struct RouterMatch<'a, Req, Res>
where
	Req: Send + Sync + 'static,
	Res: Send + Sync + 'static,
{
	pub handler: &'a dyn Route<Req, Response = Res, Future = BoxedFut<Res>>,
	pub params: RouteParams,
}

/**
# The base router

managed and matched the request path to the routes

Routes are stored as Box<Route> traits.

For each method there is a matchit router
*/
pub struct Router<Req, Res>
where
	Req: Send + Sync + 'static,
	Res: Send + Sync + 'static,
{
	get_router: matchit::Router<RouteId>,
	post_router: matchit::Router<RouteId>,
	put_router: matchit::Router<RouteId>,
	patch_router: matchit::Router<RouteId>,
	delete_router: matchit::Router<RouteId>,
	options_router: matchit::Router<RouteId>,
	head_router: matchit::Router<RouteId>,
	connect_router: matchit::Router<RouteId>,
	trace_router: matchit::Router<RouteId>,

	latest_route_id: RouteId,
	prefix: String,
	route_404: Box<dyn Route<Req, Response = Res, Future = BoxedFut<Res>>>,

	routes_get: RouteVec<Req, Res>,
	routes_post: RouteVec<Req, Res>,
	routes_put: RouteVec<Req, Res>,
	routes_patch: RouteVec<Req, Res>,
	routes_delete: RouteVec<Req, Res>,
	routes_options: RouteVec<Req, Res>,
	routes_head: RouteVec<Req, Res>,
	routes_connect: RouteVec<Req, Res>,
	routes_trace: RouteVec<Req, Res>,
}

impl<Req, Res> Router<Req, Res>
where
	Req: Send + Sync,
	Res: Send + Sync,
{
	pub fn new<S>(route_404: S) -> Self
	where
		S: Service<Req, Output = Res>,
	{
		Self {
			get_router: matchit::Router::<RouteId>::new(),
			post_router: matchit::Router::<RouteId>::new(),
			put_router: matchit::Router::<RouteId>::new(),
			patch_router: matchit::Router::<RouteId>::new(),
			delete_router: matchit::Router::<RouteId>::new(),
			options_router: matchit::Router::<RouteId>::new(),
			head_router: matchit::Router::<RouteId>::new(),
			connect_router: matchit::Router::<RouteId>::new(),
			trace_router: matchit::Router::<RouteId>::new(),
			prefix: "".to_string(),
			latest_route_id: 0,
			route_404: Box::new(GramRoute::new(route_404)),

			routes_get: vec![],
			routes_post: vec![],
			routes_put: vec![],
			routes_patch: vec![],
			routes_delete: vec![],
			routes_options: vec![],
			routes_head: vec![],
			routes_connect: vec![],
			routes_trace: vec![],
		}
	}

	/**
	# save a new route as Box

	For the given method.
	Save the route in a hash map by their id
	*/
	pub fn insert<S: 'static>(&mut self, method: Method, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		let path = self.prefix.to_string() + path;

		let (router, used_map) = match method {
			Method::GET => (&mut self.get_router, &mut self.routes_get),
			Method::POST => (&mut self.post_router, &mut self.routes_post),
			Method::PUT => (&mut self.put_router, &mut self.routes_put),
			Method::DELETE => (&mut self.delete_router, &mut self.routes_delete),
			Method::PATCH => (&mut self.patch_router, &mut self.routes_patch),
			Method::HEAD => (&mut self.head_router, &mut self.routes_head),
			Method::OPTIONS => (&mut self.options_router, &mut self.routes_options),
			Method::CONNECT => (&mut self.connect_router, &mut self.routes_connect),
			Method::TRACE => (&mut self.trace_router, &mut self.routes_trace),
			_ => panic!("wrong http method"),
		};

		router.insert(path, self.latest_route_id).unwrap();

		used_map.insert(self.latest_route_id, Box::new(route));

		self.latest_route_id += 1;
	}

	/**
	# Save a get route
	which is only matched by a get request
	*/
	pub fn get<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::GET, path, route)
	}

	/**
	# Save a post route
	which is only matched by a post request
	 */
	pub fn post<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::POST, path, route)
	}

	/**
	# Save a put route
	which is only matched by a put request
	 */
	pub fn put<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::PUT, path, route)
	}

	/**
	# Save a patch route
	which is only matched by a patch request
	 */
	pub fn patch<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::PATCH, path, route)
	}

	/**
	# Save a delete route
	which is only matched by a get delete
	 */
	pub fn delete<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::DELETE, path, route)
	}

	/**
	# Save an options route
	which is only matched by an options request
	 */
	pub fn options<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::OPTIONS, path, route)
	}

	/**
	# Save a head route
	which is only matched by a head request
	 */
	pub fn head<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::HEAD, path, route)
	}

	/**
	# Save a trace route
	which is only matched by a trace request
	 */
	pub fn trace<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::TRACE, path, route)
	}

	/**
	# Save a get connect
	which is only matched by a connect request
	 */
	pub fn connect<S>(&mut self, path: &str, route: GramRoute<S, Req, Res>)
	where
		S: Service<Req, Output = Res>,
	{
		self.insert(Method::CONNECT, path, route)
	}

	pub(crate) fn handle_req(&self, method: &Method, path: &str) -> RouterMatch<'_, Req, Res>
	{
		//map again because this time we don't need mut ref
		let (router, map) = match *method {
			Method::GET => (&self.get_router, &self.routes_get),
			Method::POST => (&self.post_router, &self.routes_post),
			Method::PUT => (&self.put_router, &self.routes_put),
			Method::DELETE => (&self.delete_router, &self.routes_delete),
			Method::PATCH => (&self.patch_router, &self.routes_patch),
			Method::HEAD => (&self.head_router, &self.routes_head),
			Method::OPTIONS => (&self.options_router, &self.routes_options),
			Method::CONNECT => (&self.connect_router, &self.routes_connect),
			Method::TRACE => (&self.trace_router, &self.routes_trace),
			_ => {
				return RouterMatch {
					handler: &*self.route_404,
					params: RouteParams::new(),
				}
			},
		};

		match router.at(path) {
			Ok(r) => {
				let params: RouteParams = r.params.into();

				if let Some(route) = map.get(*r.value) {
					RouterMatch {
						handler: &**route,
						params,
					}
				} else {
					RouterMatch {
						handler: &*self.route_404,
						params: RouteParams::new(),
					}
				}
			},
			Err(_e) => {
				RouterMatch {
					handler: &*self.route_404,
					params: RouteParams::new(),
				}
			},
		}
	}
}

#[cfg(test)]
mod test
{
	use std::sync::Arc;

	use futures::StreamExt;

	use super::*;
	use crate::{r, Request, Response};

	async fn test_handler(_req: Request) -> String
	{
		"test".to_string()
	}

	async fn test_handler_param(req: Request) -> String
	{
		let params = req.extensions().get::<RouteParams>().unwrap();

		format!("test_param: {}", params.get("id").unwrap())
	}

	async fn test_handler_all(_req: Request) -> String
	{
		"test_all".to_string()
	}

	async fn test_handler_result(_req: Request) -> Result<String, String>
	{
		Ok("test".to_string())
	}

	pub struct TestMw<S>
	{
		inner: Arc<S>,
	}

	impl<S> Service<Request> for TestMw<S>
	where
		S: Service<Request, Output = Response>,
	{
		type Output = S::Output;
		type Future = impl Future<Output = Self::Output>;

		fn call(&self, req: Request) -> Self::Future
		{
			let next = self.inner.clone();

			async move {
				//do something before req

				next.call(req).await

				//do something after
			}
		}
	}

	fn test_mw_transform<S>(s: S) -> TestMw<S>
	{
		TestMw {
			inner: Arc::new(s),
		}
	}

	#[tokio::test]
	async fn test_adding_routes_and_match()
	{
		let mut router: Router<Request, Response> = Router::new(|_req: Request| async { "404".to_string() });

		router.get("/test", r(test_handler).add(test_mw_transform));

		router.get("/test/:id", r(test_handler_param));

		router.get("/test/all/*a", r(test_handler_all));

		//match all
		let handler = router.handle_req(&Method::GET, "/test/all/abcdefg");

		let res = handler
			.handler
			.invoke(Request::new(hyper::Body::from("")))
			.await;

		let res_body = res.into_body().next().await.unwrap().unwrap();

		assert_eq!(res_body, "test_all");
		assert_eq!("abcdefg", handler.params.get("a").unwrap());

		//match with url param
		let handler = router.handle_req(&Method::GET, "/test/abcdefg");

		let mut req = Request::new(hyper::Body::from(""));
		req.extensions_mut().insert(handler.params);

		let res = handler.handler.invoke(req).await;

		let res_body = res.into_body().next().await.unwrap().unwrap();

		assert_eq!(res_body, "test_param: abcdefg");
	}

	#[tokio::test]
	async fn test_result_handler()
	{
		let mut router: Router<Request, Response> = Router::new(|_req: Request| async { "404".to_string() });

		router.get("/test", r(test_handler_result));

		router.get("/test/:id", r(test_handler_result));

		router.get("/test/all/*a", r(test_handler_result));

		let handler = router.handle_req(&Method::GET, "/test/all/abcdefg");

		let res = handler
			.handler
			.invoke(Request::new(hyper::Body::from("")))
			.await;

		let res_body = res.into_body().next().await.unwrap().unwrap();

		assert_eq!(res_body, "test");
		assert_eq!("abcdefg", handler.params.get("a").unwrap());
	}
}
