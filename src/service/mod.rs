use std::future::Future;

pub(crate) mod gram_error;
mod handler;

/**
# A service to execute from the framework

A service can be a request handler or a middleware.

The only purpose is to get the request and return a response.

<br>

## Impl by rustgram

#### Normal function and closure
- This functions can just be used out of the box, no change is needed.
- A function can return a Response, String, &'static str, Result<String, GramStdHttpErr> or Result<String, E>
- Automatic error handling:
	- when returning an Err(GramStdHttpErr) this is turned into a Response with an http error code.
	- when returning Err(E) where E impl GramHttpErr, then a custom Error response can be created and get returned by the function
	- see gram_err for more details

*/
pub trait Service<R>: Send + Sync + 'static
{
	type Output;
	type Future: Future<Output = Self::Output> + Send;

	fn call(&self, req: R) -> Self::Future;
}

/**
# Get a service and returned a new service

This is sued to build the middleware stack. With routes via the add() fn.

Everytime when adding a new middleware to the route this function gets executed.

A Middleware should hold the next (inner) service and should call it when the middleware is done processing the request.
The inner service gets passed into inner parameter so the Middleware can obtain the service.
*/
pub trait ServiceTransform<S>: Send + Sync + 'static
{
	type Service;

	fn transform(&self, inner: S) -> Self::Service;
}

/**
# Trait to turn a return into a Response

This is sued by the internally implemented Service for functions.

Supported:
- Hyper Response
- String
- &'static str
- Result<String, GramStdHttpErr>
- Result<String, E>
*/
pub trait IntoResponse<Res>
{
	fn into_response(self) -> Res;
}

pub trait HttpResult<Res>
{
	fn get_res(self) -> Res;
}
