use vine_parse::parse_expression;

fn main() {
    println!(
        "{:#?}",
        parse_expression(
            "(
                a = 10
                    + 20
            )"
        )
    )
}
