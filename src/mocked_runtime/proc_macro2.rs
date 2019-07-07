use std::iter;
use std::str::FromStr;
use std::iter::{FromIterator, Extend};

pub use crate::proc_macro2::{Spacing, Delimiter};

#[derive(Clone, Copy)]
pub struct Span(i32);

impl Span {
    pub fn call_site() -> Self {
        Span(0)
    }

    pub fn different() -> Self {
        Span(1)
    }

    pub fn eq(&self, another: &Span) -> bool {
        self.0 == another.0
    }
}

#[derive(Clone)]
pub struct TokenStream(Vec<TokenTree>);

impl TokenStream {
    pub fn new() -> Self {
        TokenStream(vec![])
    }
}

impl IntoIterator for TokenStream {
    type Item = TokenTree;
    type IntoIter = <Vec<TokenTree> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<I>(iter: I) -> Self where I: IntoIterator<Item=TokenTree> {
        Self(iter.into_iter().collect())
    }
}

impl Extend<TokenTree> for TokenStream {
    fn extend<I: IntoIterator<Item=TokenTree>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

#[derive(Clone)]
pub enum TokenTree {
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
    Group(Group),
}

impl TokenTree {
    pub fn set_span(&mut self, span: Span) {
        match self {
            TokenTree::Ident(ident) => ident.set_span(span),
            TokenTree::Punct(punct) => punct.set_span(span),
            TokenTree::Literal(literal) => literal.set_span(span),
            TokenTree::Group(group) => group.set_span(span),
        }
    }
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Ident(ident) => ident.span(),
            TokenTree::Punct(punct) => punct.span(),
            TokenTree::Literal(literal) => literal.span(),
            TokenTree::Group(group) => group.span(),
        }
    }
}

#[derive(Clone)]
pub struct Ident {
    stringed: String,
    span: Span,
}

impl Ident {
    pub fn new(string: &str, span: Span) -> Self {
        Self {
            stringed: string.into(),
            span,
        }
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn set_span(&mut self, span: Span) {
        self.span = span
    }
}

#[derive(Clone)]
pub struct Punct {
    op: char,
    spacing: Spacing,
    span: Span,
}

impl Punct {
    pub fn new(op: char, spacing: Spacing) -> Self {
        Self {
            op,
            spacing,
            span: Span::call_site(),
        }
    }
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn set_span(&mut self, span: Span) {
        self.span = span
    }
}

#[derive(Clone)]
pub struct Literal {
    stringed: String,
    span: Span,
}

impl Literal {
    pub fn span(&self) -> Span {
        self.span
    }
    pub fn set_span(&mut self, span: Span) {
        self.span = span
    }
}

impl FromStr for TokenStream {
    type Err = ();
    fn from_str(s: &str) -> Result<TokenStream, ()> {
        Ok(TokenStream::from_iter(iter::once(TokenTree::Literal(Literal {
            stringed: s.into(),
            span: Span::call_site()
        }))))
    }
}

#[derive(Clone)]
pub struct Group {
    delimiter: Delimiter,
    stream: TokenStream,
    span: Span,
}

impl Group {
    pub fn new(delimiter: Delimiter, stream: TokenStream) -> Self {
        Self {
            delimiter,
            stream,
            span: Span::call_site(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }
    pub fn set_span(&mut self, span: Span) {
        self.span = span
    }

    pub fn stream(&self) -> TokenStream {
        self.stream.clone()
    }
}
