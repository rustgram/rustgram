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
		write!(
			f,
			"Http Error with status code: {} and Message: {}",
			self.status, self.msg
		)
	}
}

impl error::Error for GramStdHttpErr {}
