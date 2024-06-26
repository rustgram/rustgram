use rustgram::route_parser;

#[test]
fn build_test_routes()
{
	route_parser::start("tests/test_routes.yml".to_string(), "tests/output.txt".to_string());
}
