use std::collections::HashMap;

use crate::type_tree::{Ident, Type};

#[derive(Debug, Clone, Default)]
pub struct Scope<'a> {
    vars: HashMap<String, Type>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter_scope(&'a self) -> Scope<'a> {
        Scope {
            vars: HashMap::new(),
            parent: Some(self),
        }
    }

    pub fn set(&mut self, ident: &str, ty: Type) {
        if self.vars.insert(ident.to_owned(), ty).is_some() {
            todo!()
        }
    }

    pub fn get(&self, ident: &Ident) -> Option<&Type> {
        self.vars
            .get(ident.name)
            .or_else(|| self.parent.and_then(|p| p.get(ident)))
    }
}
