use std::collections::HashMap;
use std::marker::PhantomData;

use async_trait::async_trait;

use crate::service::{Service, ServiceTransform};

#[async_trait]
pub(crate) trait Route<Req>: Send + Sync
{
	type Response;

	async fn invoke(&self, req: Req) -> Self::Response;
}

//__________________________________________________________________________________________________

pub struct GramRoute<S: 'static, Req, Res>
where
	S: Service<Req, Response = Res>,
	Req: Send + Sync,
	Res: Send + Sync,
{
	handler: S,
	_req: PhantomData<Req>,
	_res: PhantomData<Res>,
}

impl<S: 'static, Req, Res> GramRoute<S, Req, Res>
where
	S: Service<Req, Response = Res>,
	Req: Send + Sync,
	Res: Send + Sync,
{
	pub fn new(service: S) -> Self
	{
		return Self {
			handler: service,
			_req: Default::default(),
			_res: Default::default(),
		};
	}

	pub fn add<S1, T, Req1, Res1>(self, middleware: T) -> GramRoute<S1, Req1, Res1>
	where
		T: ServiceTransform<S, Service = S1> + Send + Sync + 'static,
		S1: Service<Req1, Response = Res1> + Send + Sync + 'static,
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

#[async_trait]
impl<S: 'static, Req, Res> Route<Req> for GramRoute<S, Req, Res>
where
	S: Service<Req, Response = Res>,
	Req: Send + Sync,
	Res: Send + Sync,
{
	type Response = Res;

	async fn invoke(&self, req: Req) -> Self::Response
	{
		self.handler.call(req).await
	}
}

//__________________________________________________________________________________________________

/**
# Returns a new GramRoute with the service
*/
pub fn r<S, Req, Res>(service: S) -> GramRoute<S, Req, Res>
where
	S: Service<Req, Response = Res>,
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
