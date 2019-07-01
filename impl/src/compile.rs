use proc_macro2::{TokenStream, Ident, Span};

use crate::language::*;
use proc_quote::quote;

pub fn compile(mquote: MQuote) -> TokenStream {
    let mut gen = IdentGenerator::new();
    compile_with(mquote, &mut gen)
}

fn compile_with(mquote: MQuote, gen: &mut IdentGenerator) -> TokenStream {
    match mquote {
        MQuote::Binding(bindings) => compile_bindings(bindings, gen),
    }
}

fn compile_bindings(bindings: MQuoteBinding, gen: &mut IdentGenerator) -> TokenStream {
    let ExpandedBindings{cons, start, bindings} = expand_bindings(bindings, gen);
    let cons = cons.into_iter()
        .map(|(var, plain)| quote!(##var #plain));
    quote!{{
        #bindings
        quote!( #start #(#cons)* )
    }}
}

struct ExpandedBindings {
    bindings: TokenStream,
    start: TokenStream,
    cons: Vec<(Ident, TokenStream)>,
}

fn expand_bindings(bindings: MQuoteBinding, gen: &mut IdentGenerator) -> ExpandedBindings {
    let (let_bindings, cons): (Vec<_>, Vec<_>) = bindings.parts.into_iter()
        .map(|(bind_with, tokens)| {
            let (bindings, var_name) = expand_binding(bind_with, gen);
            (bindings, (var_name, tokens))})
        .unzip();
    let mut all_bindings = TokenStream::new();
    all_bindings.extend(let_bindings);
    ExpandedBindings{ bindings: all_bindings, start: bindings.start, cons }
}

fn expand_binding(binding: BindWith, gen: &mut IdentGenerator) -> (TokenStream, Ident) {
    let var_name = gen.generate(&binding);
    let expr = match binding {
        BindWith::MQuote(to_be_compiled) => compile_with(*to_be_compiled, gen),
        BindWith::Expression(e) => e,
    };
    let tokens = quote!(let #var_name = #expr;);
    (tokens, var_name)
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
