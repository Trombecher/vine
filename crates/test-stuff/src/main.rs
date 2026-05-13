use vine_parse::parse_expression;

fn main() {
    println!("{:#?}", parse_expression("if True then 10 else False"))
}
