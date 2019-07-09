use proc_macro2::{TokenStream, TokenTree, Delimiter, Span};

use crate::buffer::QTokens;

pub enum TokenTreeQ {
    Insertion(Span, TokenStream),
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
    pub condition: TokenStream,
    pub then: QTokens,
    pub else_: Option<QTokens>,
}

pub struct MQuoteFor {
    pub span: Span,
    pub over: TokenStream,
    pub body: QTokens,
}

pub struct MQuoteMatch {
    pub span: Span,
    pub of: TokenStream,
    pub patterns: Vec<(Span, TokenStream, QTokens)>,
}
