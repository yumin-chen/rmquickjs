use alloc::{boxed::Box, vec::Vec};

use crate::{Context, Result, Value};

#[derive(Default)]
pub struct Opaque {
    pub interrupt_handler: Option<Box<dyn Fn(&Context) -> bool>>,
    pub funcs: Vec<Box<dyn Fn(&Context, Option<Value>, &[Value]) -> Result<Value>>>,
}
