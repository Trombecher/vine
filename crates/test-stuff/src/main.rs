use vine_parse::parse_expression;

fn main() {
    println!("{:#?}", parse_expression("1 < 3 == 3 >= 20"))
}
