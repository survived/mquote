use proc_macro2::TokenStream;

pub enum MQuote {
    Binding(MQuoteBinding),
    If(MQuoteIf),
}

pub struct MQuoteBinding {
    pub start: TokenStream,
    pub cons: Vec<(BindWith, TokenStream)>,
}

pub enum BindWith {
    MQuote(Box<MQuote>),
    Expression(TokenStream),
}

pub struct MQuoteIf {
    pub condition: TokenStream,
    pub then: Box<MQuote>,
    pub else_: Option<Box<MQuote>>,
}