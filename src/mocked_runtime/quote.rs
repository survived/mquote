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

impl<T: ToTokens+?Sized> ToTokens for &T {
    fn to_tokens(&self, token_stream: &mut TokenStream) {
        <T as ToTokens>::to_tokens(self, token_stream)
    }
}
