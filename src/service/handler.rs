use std::future::Future;

use async_trait::async_trait;
use hyper::StatusCode;

use crate::service::gram_error::{GramHttpErr, GramStdHttpErr};
use crate::service::{IntoResponse, Service, ServiceTransform};
use crate::{Request, Response};

//__________________________________________________________________________________________________
//framework specific function implementation

#[async_trait]
impl<R: Send + Sync + 'static, F: Send + Sync + 'static, Fut> Service<R> for F
where
	F: Fn(R) -> Fut,
	Fut: Future + Send + 'static,
	Fut::Output: IntoResponse<Response>,
{
	type Response = Response;

	async fn call(&self, req: R) -> Self::Response
	{
		(self)(req).await.into_response()
	}
}

impl<S, F: Send + Sync + 'static, S1: Send + Sync + 'static> ServiceTransform<S> for F
where
	S: Service<Request, Response = Response>, //define the return types from the next service
	F: Fn(S) -> S1,
	S1: Service<Request, Response = Response>,
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
