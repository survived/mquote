use std::iter;

use proc_macro2::{TokenTree, Delimiter, Span};

use crate::buffer::QTokens;

/// During the parsing TokenStream is frequently being converted into iterator. So in optimization
/// purpose in language models use already converted TokenStream.
pub type TokenIterator = iter::Peekable<<proc_macro2::TokenStream as IntoIterator>::IntoIter>;

pub enum TokenTreeQ {
    Insertion(Span, TokenIterator),
    If(MQuoteIf),
    For(MQuoteFor),
    Match(MQuoteMatch),
    Group(MQuoteGroup),
    Plain(TokenTree),
}

impl TokenTreeQ {
    pub fn span(&self) -> Span {
        match self {
            TokenTreeQ::Insertion(span, _) => *span,
            TokenTreeQ::If(MQuoteIf{ span, .. }) => *span,
            TokenTreeQ::For(MQuoteFor{ span, .. }) => *span,
            TokenTreeQ::Match(MQuoteMatch{ span, .. }) => *span,
            TokenTreeQ::Group(group) => group.span,
            TokenTreeQ::Plain(token) => token.span(),
        }
    }
}

pub struct MQuoteGroup {
    pub delimiter: Delimiter,
    pub tokens: QTokens,
    pub span: Span,
}

pub struct MQuoteIf {
    pub span: Span,
    pub condition: TokenIterator,
    pub then: QTokens,
    pub else_: Option<QTokens>,
}

pub struct MQuoteFor {
    pub span: Span,
    pub over: TokenIterator,
    pub body: QTokens,
}

pub struct MQuoteMatch {
    pub span: Span,
    pub of: TokenIterator,
    pub patterns: Vec<(Span, TokenIterator, QTokens)>,
}
