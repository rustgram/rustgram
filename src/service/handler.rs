use std::future::Future;
use std::pin::Pin;

use hyper::StatusCode;

use crate::service::gram_error::{GramHttpErr, GramStdHttpErr};
use crate::service::{HttpResult, IntoResponse, Service, ServiceTransform};
use crate::{Request, Response};

//__________________________________________________________________________________________________
//framework specific function implementation

impl<R: Send, F, Fut> Service<R> for F
where
	F: Fn(R) -> Fut + Send + Sync + 'static,
	Fut: Future + Send + 'static,
	Fut::Output: IntoResponse<Response>,
{
	type Output = Response;
	type Future = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

	fn call(&self, req: R) -> Self::Future
	{
		let res = (self)(req);

		Box::pin(async move {
			//future
			res.await.into_response()
		})
	}
}

impl<'a, S, F: Send + Sync + 'static, S1: Send + Sync + 'static> ServiceTransform<S> for F
where
	S: Service<Request, Output = Response> + Send, //define the return types from the next service
	F: Fn(S) -> S1,
	S1: Service<Request, Output = Response> + Send,
{
	type Service = S1;

	fn transform(&self, inner: S) -> Self::Service
	{
		(self)(inner)
	}
}

//__________________________________________________________________________________________________
//framework specific into res implementation

impl IntoResponse<Response> for Response
{
	fn into_response(self) -> Response
	{
		self
	}
}

impl IntoResponse<Response> for &'static str
{
	fn into_response(self) -> Response
	{
		Response::new(self.into())
	}
}

impl IntoResponse<Response> for String
{
	fn into_response(self) -> Response
	{
		Response::new(self.into())
	}
}

impl IntoResponse<Response> for Result<String, GramStdHttpErr>
{
	fn into_response(self) -> Response
	{
		return match self {
			Ok(str) => str.into_response(),
			Err(e) => handle_gram_err(e),
		};
	}
}

impl<E> IntoResponse<Response> for Result<String, E>
where
	E: GramHttpErr<Response>,
{
	fn into_response(self) -> Response
	{
		match self {
			Ok(s) => s.into_response(),
			Err(e) => e.get_res(),
		}
	}
}

impl<R, E> IntoResponse<Response> for Result<R, E>
where
	R: HttpResult<Response>,
	E: GramHttpErr<Response>,
{
	fn into_response(self) -> Response
	{
		match self {
			Ok(s) => s.get_res(),
			Err(e) => e.get_res(),
		}
	}
}

fn handle_gram_err(e: GramStdHttpErr) -> Response
{
	let status = match StatusCode::from_u16(e.status) {
		Ok(s) => s,
		Err(_e) => StatusCode::BAD_REQUEST,
	};

	return hyper::Response::builder()
		.status(status)
		.body(hyper::Body::from(e.msg))
		.unwrap();
}
