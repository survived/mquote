use std::iter::FromIterator;

use proc_macro2::TokenTree;

use crate::language::*;

pub struct QTokens {
    tokens: Vec<TokenTreeQ>,
}

impl QTokens {
    pub fn new() -> Self {
        Self { tokens: vec![] }
    }
    pub fn push(&mut self, token: TokenTreeQ) {
        self.tokens.push(token);
    }
}

impl IntoIterator for QTokens {
    type Item = TokenTreeQ;
    type IntoIter = <Vec<TokenTreeQ> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.tokens.into_iter()
    }
}

impl FromIterator<TokenTreeQ> for QTokens {
    fn from_iter<T: IntoIterator<Item=TokenTreeQ>>(iter: T) -> Self {
        Self {
            tokens: iter.into_iter().collect(),
        }
    }
}

impl FromIterator<TokenTree> for QTokens {
    fn from_iter<T: IntoIterator<Item=TokenTree>>(iter: T) -> Self {
        Self {
            tokens: iter.into_iter().map(TokenTreeQ::Plain).collect(),
        }
    }
}

impl From<Vec<TokenTreeQ>> for QTokens {
    fn from(tokens: Vec<TokenTreeQ>) -> Self {
        Self {
            tokens,
        }
    }
}
