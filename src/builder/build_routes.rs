use crate::builder::entities::{Config, GramRoute, Mw, Route, G};

pub(crate) fn start_routes_build(config: &GramRoute) -> Vec<Route>
{
	let mut routes = Vec::new();

	let prefix = config.prefix.clone();

	for c in &config.routes {
		match c {
			Config::G(g) => {
				let route = handle_group(g, prefix.clone(), &config.mw);

				for r in route {
					routes.push(r);
				}
			},
			e => {
				let route = handle_route(e, prefix.clone(), &config.mw);
				routes.push(route);
			},
		}
	}

	routes
}

fn handle_route(route: &Config, prefix: String, g_mw: &Mw) -> Route
{
	let path;
	let method;
	let s;
	let mw;

	match route {
		Config::Get(r) => {
			path = prefix + &*r.p;
			mw = r.mw.clone();
			s = r.s.clone();
			method = "get";
		},
		Config::Post(r) => {
			path = prefix + &*r.p;
			mw = r.mw.clone();
			s = r.s.clone();
			method = "post";
		},
		Config::Put(r) => {
			path = prefix + &*r.p;
			mw = r.mw.clone();
			s = r.s.clone();
			method = "put";
		},
		Config::Delete(r) => {
			path = prefix + &*r.p;
			mw = r.mw.clone();
			s = r.s.clone();
			method = "delete";
		},
		Config::Head(r) => {
			path = prefix + &*r.p;
			mw = r.mw.clone();
			s = r.s.clone();
			method = "head";
		},
		Config::Options(r) => {
			path = prefix + &*r.p;
			mw = r.mw.clone();
			s = r.s.clone();
			method = "options";
		},
		Config::Patch(r) => {
			path = prefix + &*r.p;
			mw = r.mw.clone();
			s = r.s.clone();
			method = "patch";
		},
		_ => {
			panic!("route not found");
		},
	}

	let mut string = format!("r({})", s);

	//handle mw
	//first apply the route mw

	if let Some(mw) = mw {
		for r_mw_i in mw {
			string += &*format!(".add({})", r_mw_i);
		}
	}

	//2nd apply group mw
	if let Some(mw) = g_mw {
		for g_mw_i in mw {
			string += &*format!(".add({})", g_mw_i);
		}
	}

	Route {
		path,
		route: string,
		method,
	}
}

fn handle_group(group: &G, prefix: String, parent_mw: &Mw) -> Vec<Route>
{
	let mut mw = Vec::new();

	//apply first the group mw from this group, and then from the parent
	if let Some(m) = group.mw.clone() {
		//the group got middleware
		for g_mw in m {
			mw.push(g_mw);
		}
	}

	if let Some(m) = parent_mw {
		for p_mw in m {
			mw.push(p_mw.clone());
		}
	}

	let mw = Some(mw);

	let prefix = prefix + &*group.p;

	let mut routes = Vec::new();

	for gr in &group.gr {
		match gr {
			Config::G(g) => {
				let group = handle_group(g, prefix.clone(), &mw);

				for g in group {
					routes.push(g);
				}
			},
			e => {
				let route = handle_route(e, prefix.clone(), &mw);
				routes.push(route);
			},
		}
	}

	routes
}
