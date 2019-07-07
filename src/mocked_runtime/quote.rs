use std::iter;

use super::proc_macro2::{TokenStream, TokenTree, Ident};

pub trait ToTokens {
    fn to_tokens(&self, token_stream: &mut TokenStream);
}

impl ToTokens for Ident {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        token_stream.extend(iter::once(TokenTree::Ident(self.clone())))
    }
}
