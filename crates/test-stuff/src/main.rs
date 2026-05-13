use vine_parse::parse_expression;

fn main() {
    println!("{:#?}", parse_expression("1 + 2 == 4 and 1 < 3"))
}
