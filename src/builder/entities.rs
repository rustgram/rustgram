use serde::{Deserialize, Serialize};

pub(crate) type Mw = Option<Vec<String>>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "get"))]
pub(crate) struct Get
{
	pub(crate) p: String,
	pub(crate) s: String,
	pub(crate) mw: Mw,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "post"))]
pub(crate) struct Post
{
	pub(crate) p: String,
	pub(crate) s: String,
	pub(crate) mw: Mw,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "put"))]
pub(crate) struct Put
{
	pub(crate) p: String,
	pub(crate) s: String,
	pub(crate) mw: Mw,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "head"))]
pub(crate) struct Head
{
	pub(crate) p: String,
	pub(crate) s: String,
	pub(crate) mw: Mw,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "options"))]
pub(crate) struct Options
{
	pub(crate) p: String,
	pub(crate) s: String,
	pub(crate) mw: Mw,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "delete"))]
pub(crate) struct Delete
{
	pub(crate) p: String,
	pub(crate) s: String,
	pub(crate) mw: Mw,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "patch"))]
pub(crate) struct Patch
{
	pub(crate) p: String,
	pub(crate) s: String,
	pub(crate) mw: Mw,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename(deserialize = "group"))]
pub(crate) struct G
{
	pub(crate) p: String,
	pub(crate) mw: Mw,
	pub(crate) gr: Vec<Config>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Config
{
	#[serde(rename(deserialize = "get"))]
	Get(Get),
	#[serde(rename(deserialize = "post"))]
	Post(Post),
	#[serde(rename(deserialize = "put"))]
	Put(Put),
	#[serde(rename(deserialize = "delete"))]
	Delete(Delete),
	#[serde(rename(deserialize = "options"))]
	Options(Options),
	#[serde(rename(deserialize = "head"))]
	Head(Head),
	#[serde(rename(deserialize = "patch"))]
	Patch(Patch),
	#[serde(rename(deserialize = "group"))]
	G(G),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GramRoute
{
	pub(crate) base_handler: String,
	pub(crate) base_mw: String,
	pub(crate) prefix: String,
	pub(crate) handler_404: Option<String>,
	pub(crate) mw: Mw,
	pub(crate) routes: Vec<Config>,
}

//config structs

#[derive(Debug)]
pub(crate) struct Route
{
	pub(crate) path: String,
	pub(crate) route: String,
	pub(crate) method: &'static str,
}
