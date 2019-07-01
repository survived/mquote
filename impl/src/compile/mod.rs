use proc_macro2::{TokenStream, Ident, Span};

use crate::language::{MQuote, BindWith};

mod binding;
mod if_expr;

pub fn compile(mquote: MQuote) -> TokenStream {
    let mut gen = IdentGenerator::new();
    compile_with(mquote, &mut gen)
}

fn compile_with(mquote: MQuote, gen: &mut IdentGenerator) -> TokenStream {
    match mquote {
        MQuote::Binding(bindings) => binding::compile(bindings, gen),
        MQuote::If(if_expr) => if_expr::compile(if_expr, gen),
    }
}

struct IdentGenerator {
    i: usize,
}

impl IdentGenerator {
    pub fn new() -> Self {
        Self{ i: 0 }
    }

    pub fn generate(&mut self, _hint: &BindWith) -> Ident {
        let i = self.i;
        self.i += 1;
        let var_name = format!("__mquote{}", i);
        Ident::new(&var_name, Span::call_site())
    }
}
