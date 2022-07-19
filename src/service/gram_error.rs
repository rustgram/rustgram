use std::error;
use std::fmt::{Display, Formatter};

/**
# A pre defined lib error

This error will be handled by the framework. No extra error trait is needed

Example:
```rust
use rustgram::{GramStdHttpErr, Request};

pub async fn test_handler_err(_req: Request) -> Result<String, GramStdHttpErr>
{
	Err(GramStdHttpErr::new(400,format!("Bad Request")))
}
```
*/
#[derive(Debug)]
pub struct GramStdHttpErr
{
	pub status: u16,
	pub msg: String,
}

impl GramStdHttpErr
{
	pub fn new(status: u16, msg: String) -> Self
	{
		Self {
			status,
			msg,
		}
	}
}

impl Display for GramStdHttpErr
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "Http Error with status code: {} and Message: {}", self.status, self.msg)
	}
}

impl error::Error for GramStdHttpErr {}

/**
# Trait for custom error types

This is useful if the normal GramStdHttpErr is not enough
e.g. for different response header when returning json error.

Example:
```rust
use rustgram::{GramStdHttpErr, Request};

pub async fn test_handler_err(_req: Request) -> Result<String, GramStdHttpErr>
{
	Err(GramStdHttpErr::new(400,format!("Bad Request")))
}
```
*/
pub trait GramHttpErr<Res>
{
	/**
	The Response must be created here.

	Example:
	```rust
	use hyper::StatusCode;
	use rustgram::{GramHttpErr, Response, Request};

	pub struct HttpErr
	{
		http_status_code: u16,
		api_error_code: u32,
		msg: &'static str,
	}

	impl HttpErr
	{
		pub fn new(http_status_code: u16, api_error_code: u32, msg: &'static str) -> Self
		{
			Self {
				http_status_code,
				api_error_code,
				msg
			}
		}
	}

	impl GramHttpErr<Response> for HttpErr
	{
		fn get_res(&self) -> Response
		{
			let status = match StatusCode::from_u16(self.http_status_code) {
				Ok(s) => s,
				Err(_e) => StatusCode::BAD_REQUEST,
			};

			//the msg for the end user
			let msg = format!(
				"{{\"status\": {}, \"error_message\": \"{}\"}}",
				self.api_error_code, self.msg);

			hyper::Response::builder()
				.status(status)
				.header("Content-Type", "application/json")
				.body(hyper::Body::from(msg))
				.unwrap()
		}
	}

	//example usage:

	pub async fn test_handler_err(_req: Request) -> Result<String, HttpErr>
	{
		Err(HttpErr::new(400,1,"Input not valid"))
	}
	```
	*/
	fn get_res(&self) -> Res;
}
