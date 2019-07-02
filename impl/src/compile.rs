use std::iter;
use proc_macro2::{self, TokenStream, Ident, Span, TokenTree};
use proc_quote::quote;

use crate::language::*;

pub fn compile(mquote: TokenStreamQ) -> TokenStream {
    let mut gen = IdentGenerator::new();
    let mut bindings = vec![];
    let stream = compile_with(mquote, &mut gen, &mut bindings);

    let mut binding_stream = TokenStream::new();
    for (var, binding) in bindings {
        let expr = match binding {
            TokenTreeQ::Insertion(MQuoteInsertion::Unescaped(insertion)) => quote!(#insertion),
            TokenTreeQ::If(MQuoteIf{ condition, then, else_ }) => {
                let then = compile(then);
                let else_ = else_.map(compile).unwrap_or(Default::default());
                quote!{{
                    if #condition { #then } else { #else_ }
                }}
            },
            TokenTreeQ::Insertion(_) | TokenTreeQ::Group(_) | TokenTreeQ::Plain(_) =>
                unreachable!("guaranteed by process_token's match")
        };
        binding_stream.extend(iter::once(quote!(let ref #var = #expr;)))
    }

    quote!{{#binding_stream quote!(#stream)}}
}

fn compile_with(mquote: TokenStreamQ, gen: &mut IdentGenerator, bindings: &mut Vec<(Ident, TokenTreeQ)>) -> TokenStream {
    let mut stream = TokenStream::new();
    for token in mquote {
        match token {
            TokenTreeQ::Plain(token) => stream.extend(iter::once(token)),
            token => stream.extend(process_token(token, gen, bindings)),
        }
    }
    stream
}

fn process_token(token: TokenTreeQ, gen: &mut IdentGenerator, bindings: &mut Vec<(Ident, TokenTreeQ)>) -> TokenStream {
    match token {
        TokenTreeQ::Plain(token) => iter::once(token).collect(),
        TokenTreeQ::Insertion(MQuoteInsertion::Escaped(tokens))
            => quote!(##tokens),
        TokenTreeQ::Group(MQuoteGroup{ delimiter, stream, span }) => {
            let stream = compile_with(stream, gen, bindings);
            let mut group = proc_macro2::Group::new(delimiter, stream);
            group.set_span(span);
            iter::once(TokenTree::from(group)).collect()
        }
        token => {
            let var = gen.generate(&token);
            bindings.push((var.clone(), token));
            quote!(##var)
        }
    }
}

struct IdentGenerator {
    i: usize,
}

impl IdentGenerator {
    pub fn new() -> Self {
        Self{ i: 0 }
    }

    pub fn generate(&mut self, _hint: &TokenTreeQ) -> Ident {
        let i = self.i;
        self.i += 1;
        let var_name = format!("__mquote{}", i);
        Ident::new(&var_name, Span::call_site())
    }
}
