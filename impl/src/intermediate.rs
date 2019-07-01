use proc_macro::TokenStream;

pub enum MQuote {
    Binding(MQuoteBinding),
}

pub struct MQuoteBinding {
    parts: Vec<(TokenStream, BindWith)>,
}

pub enum BindWith {
    MQuote(Box<MQuote>),
    Expression(TokenStream),
}
