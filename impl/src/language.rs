use proc_macro2::{TokenStream, TokenTree, Delimiter, Span};

use crate::buffer::QTokens;

pub enum TokenTreeQ {
    Insertion(TokenStream),
    If(MQuoteIf),
    For(MQuoteFor),
    Match(MQuoteMatch),
    Group(MQuoteGroup),
    Plain(TokenTree),
}

impl TokenTreeQ {
    pub fn span(&self) -> Span {
        match self {
            TokenTreeQ::Group(group) => group.span,
            TokenTreeQ::Plain(token) => token.span(),
            TokenTreeQ::Insertion(_) | TokenTreeQ::If(_) | TokenTreeQ::For(_) | TokenTreeQ::Match(_)
                => Span::call_site(),
        }
    }
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

pub struct MQuoteFor {
    pub over: TokenStream,
    pub body: QTokens,
}

pub struct MQuoteMatch {
    pub of: TokenStream,
    pub patterns: Vec<(TokenStream, QTokens)>,
}
