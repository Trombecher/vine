use vine_parse::parse_expression;

fn main() {
    println!("{:#?}", parse_expression("function a is b => c * 2"))
}
