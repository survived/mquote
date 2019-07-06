use std::iter::{IntoIterator, FromIterator};
use proc_macro2::{TokenStream, TokenTree, Delimiter, Span};

use crate::buffer::QTokens;

pub struct TokenStreamQ(Vec<TokenTreeQ>);

pub enum TokenTreeQ {
    Insertion(TokenStream),
    If(MQuoteIf),
    Group(MQuoteGroup),
    Plain(TokenTree),
}

pub struct MQuoteGroup {
    pub delimiter: Delimiter,
    pub tokens: QTokens,
    pub span: Span,
}

pub struct MQuoteIf {
    pub condition: TokenStream,
    pub then: QTokens,
    pub else_: Option<QTokens>,
}
