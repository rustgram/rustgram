use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use hyper::Method;
use nohash_hasher::NoHashHasher;

use crate::router::route::{GramRoute, Route};
use crate::service::Service;
use crate::RouteParams;

pub mod route;

type NonHashMap<T, V> = HashMap<T, V, BuildHasherDefault<NoHashHasher<T>>>;

type RouteId = u64;

pub(crate) struct RouterMatch<'a, Req, Res>
where
	Req: Send + Sync + 'static,
	Res: Send + Sync + 'static,
{
	pub handler: &'a dyn Route<Req, Response = Res>,
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
	routes: NonHashMap<u8, NonHashMap<RouteId, Box<dyn Route<Req, Response = Res>>>>,
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
	route_404: Box<dyn Route<Req, Response = Res>>,
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
			routes: HashMap::with_hasher(BuildHasherDefault::default()),

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
		self.latest_route_id += 1;

		let (router, used_map) = match method {
			Method::GET => (&mut self.get_router, 0_u8),
			Method::POST => (&mut self.post_router, 1_u8),
			Method::PUT => (&mut self.put_router, 2_u8),
			Method::DELETE => (&mut self.delete_router, 3_u8),
			Method::PATCH => (&mut self.patch_router, 4_u8),
			Method::HEAD => (&mut self.head_router, 5_u8),
			Method::OPTIONS => (&mut self.options_router, 6_u8),
			Method::CONNECT => (&mut self.connect_router, 7_u8),
			Method::TRACE => (&mut self.trace_router, 8_u8),
			_ => panic!("wrong http method"),
		};

		router.insert(path, self.latest_route_id).unwrap();

		if self.routes.get(&used_map).is_none() {
			//init the hash map for this method
			let route_map = HashMap::with_hasher(BuildHasherDefault::default());

			//need primitive types for non hasher
			self.routes.insert(used_map, route_map);
		}

		match self.routes.get_mut(&used_map) {
			Some(m) => m.insert(self.latest_route_id, Box::new(route)),
			None => panic!("Route insert failed"),
		};
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
		let (router, used_map) = match method {
			&Method::GET => (&self.get_router, self.routes.get(&0)),
			&Method::POST => (&self.post_router, self.routes.get(&1)),
			&Method::PUT => (&self.put_router, self.routes.get(&2)),
			&Method::DELETE => (&self.delete_router, self.routes.get(&3)),
			&Method::PATCH => (&self.patch_router, self.routes.get(&4)),
			&Method::HEAD => (&self.head_router, self.routes.get(&5)),
			&Method::OPTIONS => (&self.options_router, self.routes.get(&6)),
			&Method::CONNECT => (&self.connect_router, self.routes.get(&7)),
			&Method::TRACE => (&self.trace_router, self.routes.get(&8)),
			_ => {
				return RouterMatch {
					handler: &*self.route_404,
					params: RouteParams::new(),
				}
			},
		};

		let map = match used_map {
			None => {
				return RouterMatch {
					handler: &*self.route_404,
					params: RouteParams::new(),
				}
			},
			Some(m) => m,
		};

		match router.at(path) {
			Ok(r) => {
				let params: RouteParams = r.params.into();

				if let Some(route) = map.get(&r.value) {
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
	use futures::StreamExt;

	use super::*;
	use crate::{r, Request, Response};

	async fn test_handler(_req: Request) -> String
	{
		format!("test")
	}

	async fn test_handler_param(req: Request) -> String
	{
		let params = req.extensions().get::<RouteParams>().unwrap();

		format!("test_param: {}", params.get("id").unwrap())
	}

	async fn test_handler_all(_req: Request) -> String
	{
		format!("test_all")
	}

	#[tokio::test]
	async fn test_adding_routes_and_match()
	{
		let mut router: Router<Request, Response> = Router::new(|_req: Request| async { format!("404") });

		router.get("/test", r(test_handler));

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
		assert_eq!("/abcdefg", handler.params.get("a").unwrap());

		//match with url param
		let handler = router.handle_req(&Method::GET, "/test/abcdefg");

		let mut req = Request::new(hyper::Body::from(""));
		req.extensions_mut().insert(handler.params);

		let res = handler.handler.invoke(req).await;

		let res_body = res.into_body().next().await.unwrap().unwrap();

		assert_eq!(res_body, "test_param: abcdefg");
	}
}
