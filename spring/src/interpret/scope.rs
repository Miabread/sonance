use std::collections::HashMap;

use crate::interpret::Value;

#[derive(Debug, Clone, Default)]
pub struct Scope<'src, 'scope> {
    vars: HashMap<String, Value<'src>>,
    parent: Option<&'scope Scope<'src, 'scope>>,
}

impl<'src, 'scope> Scope<'src, 'scope> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enter_scope(&'scope self) -> Scope<'src, 'scope> {
        Scope {
            vars: HashMap::new(),
            parent: Some(self),
        }
    }

    pub fn set(&mut self, ident: &str, value: Value<'src>) {
        if self.vars.insert(ident.to_owned(), value).is_some() {
            todo!()
        }
    }

    pub fn get(&self, ident: &str) -> Option<&Value<'src>> {
        self.vars
            .get(ident)
            .or_else(|| self.parent.and_then(|p| p.get(ident)))
    }
}
