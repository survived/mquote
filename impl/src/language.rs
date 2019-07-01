use proc_macro2::TokenStream;

pub enum MQuote {
    Binding(MQuoteBinding),
}

pub struct MQuoteBinding {
    pub start: TokenStream,
    pub parts: Vec<(BindWith, TokenStream)>,
}

pub enum BindWith {
    MQuote(Box<MQuote>),
    Expression(TokenStream),
}
