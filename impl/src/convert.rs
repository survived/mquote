use proc_macro2::{TokenStream, Ident, Span};

use intermediate::*;

pub fn expand_intermediate(mquote: MQuote, gen: &mut IdentGenerator) -> TokenStream {
    unimplemented!();
}


fn expand_binding(binding: MQuoteBinding, gen: &mut IdentGenerator) -> TokenStream {

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
