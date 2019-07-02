use std::iter::{IntoIterator, FromIterator};
use proc_macro2::{TokenStream, TokenTree, Delimiter, Span};

pub struct TokenStreamQ(Vec<TokenTreeQ>);

impl IntoIterator for TokenStreamQ {
    type Item = TokenTreeQ;
    type IntoIter = <Vec<TokenTreeQ> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<TokenTreeQ> for TokenStreamQ {
    fn from_iter<T: IntoIterator<Item=TokenTreeQ>>(iter: T) -> Self {
        TokenStreamQ(iter.into_iter().collect())
    }
}

impl FromIterator<TokenTree> for TokenStreamQ {
    fn from_iter<T: IntoIterator<Item=TokenTree>>(iter: T) -> Self {
        TokenStreamQ(iter.into_iter().map(TokenTreeQ::Plain).collect())
    }
}

impl From<Vec<TokenTreeQ>> for TokenStreamQ {
    fn from(vec: Vec<TokenTreeQ>) -> Self {
        Self(vec)
    }
}

pub enum TokenTreeQ {
    Insertion(MQuoteInsertion),
    If(MQuoteIf),
    Group(MQuoteGroup),
    Plain(TokenTree),
}

pub struct MQuoteGroup {
    pub delimiter: Delimiter,
    pub stream: TokenStreamQ,
    pub span: Span,
}

pub enum MQuoteInsertion {
    Escaped(TokenStream),
    Unescaped(TokenStream),
}

pub struct MQuoteIf {
    pub condition: TokenStream,
    pub then: TokenStreamQ,
    pub else_: Option<TokenStreamQ>,
}
