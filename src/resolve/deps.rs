use crate::resolve::ast::Expression;

trait Deps {
    fn get_deps(&self);
}

impl<'a> Deps for Expression<'a> {
    fn get_deps(&self) {
        
    }
}