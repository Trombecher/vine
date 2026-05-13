use std::sync::Arc;

pub enum Value<'source> {
    String(&'source str),
    Array(Arc<[u8]>),
    Function {},
}

pub enum Expression {}
