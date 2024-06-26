use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::service::{Service, ServiceTransform};

pub(crate) trait Route<Req>: Send + Sync
{
	type Response;
	type Future: Future<Output = Self::Response> + Send;

	fn invoke(&self, req: Req) -> Self::Future;
}

//__________________________________________________________________________________________________

pub struct GramRoute<S: 'static, Req, Res>
where
	S: Service<Req, Output = Res>,
	Req: Send + Sync,
	Res: Send + Sync,
{
	handler: S,
	_req: PhantomData<Req>,
	_res: PhantomData<Res>,
}

impl<S, Req, Res> GramRoute<S, Req, Res>
where
	S: Service<Req, Output = Res> + 'static + Sync,
	Req: Send + Sync,
	Res: Send + Sync,
{
	pub fn new(service: S) -> Self
	{
		Self {
			handler: service,
			_req: Default::default(),
			_res: Default::default(),
		}
	}

	pub fn add<S1, T, Req1, Res1>(self, middleware: T) -> GramRoute<S1, Req1, Res1>
	where
		T: ServiceTransform<S, Service = S1> + Send + Sync + 'static,
		S1: Service<Req1, Output = Res1>,
		Req1: Send + Sync,
		Res1: Send + Sync,
	{
		GramRoute {
			handler: middleware.transform(self.handler),
			_req: Default::default(),
			_res: Default::default(),
		}
	}
}

impl<S, Req, Res> Route<Req> for GramRoute<S, Req, Res>
where
	S: Service<Req, Output = Res>,
	Req: Send + Sync,
	Res: Send + Sync,
{
	type Response = Res;
	type Future = Pin<Box<dyn Future<Output = Self::Response> + Send>>;

	fn invoke(&self, req: Req) -> Self::Future
	{
		let res = self.handler.call(req);

		Box::pin(res)
	}
}

//__________________________________________________________________________________________________

/**
# Returns a new GramRoute with the service
*/
pub fn r<S, Req, Res>(service: S) -> GramRoute<S, Req, Res>
where
	S: Service<Req, Output = Res>,
	Req: Send + Sync,
	Res: Send + Sync,
{
	GramRoute::new(service)
}

//__________________________________________________________________________________________________

//from here: https://github.com/cloudflare/workers-rs/blob/d8b1149119ebf60fc0c2480cdf64996cfd152fac/worker/src/router.rs#L23
pub struct RouteParams(HashMap<String, String>);

impl RouteParams
{
	pub(crate) fn new() -> Self
	{
		RouteParams(HashMap::new())
	}

	pub fn get(&self, key: &str) -> Option<&String>
	{
		self.0.get(key)
	}
}

impl From<matchit::Params<'_, '_>> for RouteParams
{
	fn from(p: matchit::Params) -> Self
	{
		let mut route_params = RouteParams(HashMap::new());
		for (ident, value) in p.iter() {
			route_params.0.insert(ident.into(), value.into());
		}

		route_params
	}
}
