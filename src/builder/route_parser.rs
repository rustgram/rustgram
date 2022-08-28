use std::fs::File;
use std::io::{BufReader, Write};

use crate::builder::build_routes::start_routes_build;
use crate::builder::build_routes_file::build_routes;
use crate::builder::entities::GramRoute;

/**
# Build the route file from yml input

- groups can only be build by the route builder
- the builder parses a yml file and create a new route file. this file contains a function which returns a router (to use it later).
- all routes in a route shares the same middleware and the same prefix
- nested groups are also possible

1. Create a 2nd bin crate for the route builder.
2. This crate calls the builder function
3. Set the input and the output path (both relative to the current working directory)
4. Build and execute the route builder everytime the routes are changed
5. use the route function, from the new file, to get the router

````shell
# in a workspace just create a new crate
cargo new route_builder
````

Open the main function in `src/main.rs`

````ignore
use rustgram::route_parser;

fn main()
{
	//input path: from the root of the current working directory
	//output path: into the app crate (created via cargo new app)
	route_parser::start(
		"routes.yml".to_string(),
		"app/src/routes.rs".to_string()
	);
}
````

Create the route file in yml.

````yaml
# define the namespace where the route handlers live
# or leave it emtpy and use the full path to the handler
base_handler: test_handler
# define the namespace for the middleware
base_mw: test_mw
# the 404 handler. is called when no routes matched the given path
handler_404: crate::not_found_handler
# prefix for all routes
prefix: "/"

# the routes and groups. use the method followed by the path (p) and the handler (s)
routes:
  - get:
	  p: ""
	  # must match the base_handler namespace
	  s: test_handler::test_handler
  # a put route with middleware
  - put:
	  p: ""
	  s: test_handler::test_handler
	  mw:
		- mw1_transform
		- mw_transform
  # a group of routes.
  # a prefix (p) for all routes and middleware (mw) like routes
  - group:
	  p: admin
	  mw:
		- mw1_transform
		- mw_transform
	  # the routes to this group
	  gr:
		- get:
			p: ""
			s: test_handler_db::test_handler_db_to_json
		- get:
			p: "/user/:id"
			s: test_handler::test_handler
		# for this route, mw is called first, then mw1, mw2 and mw3
		- put:
			p: "/many_mw"
			s: test_handler::test_handler
			mw:
			  - mw3_transform
			  - mw2_transform
  - group:
	  p: nested
	  mw:
		- mw1_transform
		- mw_transform
	  gr:
		# define a new group inside the group routes
		- group:
			p: "/management"
			mw:
			  - mw1_transform
			  - mw_transform
			gr:
			  - put:
				  p: "/put"
				  s: test_handler::test_handler
				  mw:
					- mw5_transform
					- mw4_transform
````

This file is parsed to this:

````ignore
/**
# Generated route files by rustgram route builder.

Please do not modify this file. Any changes will be overridden by the next route build.
Use the returned router instead
 */
use rustgram::{r, Router};

use crate::test_handler::*;
use crate::test_mw::*;

pub(crate) fn routes() -> Router
{
	let mut router = Router::new(crate::not_found_handler);
	router.get("/", r(test_handler::test_handler));

	router.put(
		"/",
		r(test_handler::test_handler)
			.add(mw1_transform)
			.add(mw_transform),
	);

	router.get(
		"/admin",
		r(test_handler_db::test_handler_db_to_json)
			.add(mw1_transform)
			.add(mw_transform),
	);

	router.get(
		"/admin/user/:id",
		r(test_handler::test_handler)
			.add(mw1_transform)
			.add(mw_transform),
	);

	router.put(
		"/admin/many_mw",
		r(test_handler::test_handler)
			.add(mw3_transform)
			.add(mw2_transform)
			.add(mw1_transform)
			.add(mw_transform),
	);

	router.put(
		"/nested/management/put",
		r(test_handler::test_handler)
			.add(mw5_transform)
			.add(mw4_transform)
			.add(mw3_transform)
			.add(mw2_transform)
			.add(mw1_transform)
			.add(mw_transform),
	);

	router
}
````
*/
pub fn start(path_input: String, path_output: String)
{
	let file = File::open(path_input).unwrap();

	let reader = BufReader::new(file);

	let config: GramRoute = serde_yaml::from_reader(reader).unwrap();

	let routes = start_routes_build(&config);

	let file = build_routes(routes, config.base_handler, config.base_mw);

	let mut open_file = File::create(path_output).unwrap();

	open_file.write_all(file.as_ref()).expect("");
}
