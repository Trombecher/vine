use parse::ast::TypeParameters;

pub enum Item<'a> {
    Enum {
        tps: TypeParameters<'a>,
    }
}