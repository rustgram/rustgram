use std::future::Future;

use hyper::StatusCode;

use crate::service::gram_error::GramStdHttpErr;
use crate::service::{IntoResponse, Service, ServiceTransform};
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

	fn call(&self, req: R) -> impl Future<Output = Self::Output> + Send + 'static
	{
		let res = self(req);

		async move {
			//future
			res.await.into_response()
		}
	}
}

impl<S, F: Send + Sync + 'static, S1: Send + Sync + 'static> ServiceTransform<S> for F
where
	S: Service<Request, Output = Response> + Send, //define the return types from the next service
	F: Fn(S) -> S1,
	S1: Service<Request, Output = Response> + Send,
{
	type Service = S1;

	fn transform(&self, inner: S) -> Self::Service
	{
		self(inner)
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
		match self {
			Ok(str) => str.into_response(),
			Err(e) => handle_gram_err(e),
		}
	}
}

impl<R, E> IntoResponse<Response> for Result<R, E>
where
	R: IntoResponse<Response>,
	E: IntoResponse<Response>,
{
	fn into_response(self) -> Response
	{
		match self {
			Ok(s) => s.into_response(),
			Err(e) => e.into_response(),
		}
	}
}

fn handle_gram_err(e: GramStdHttpErr) -> Response
{
	let status = StatusCode::from_u16(e.status).unwrap_or(StatusCode::BAD_REQUEST);

	hyper::Response::builder()
		.status(status)
		.body(hyper::Body::from(e.msg))
		.unwrap()
}
