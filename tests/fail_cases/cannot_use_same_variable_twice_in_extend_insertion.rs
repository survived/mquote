extern crate mquote;

use mquote::mquote;

fn main() {
    let insertion = mquote!(h e l l o);
    let q = mquote!(let _ = [ ^{insertion} ^{insertion} ]);
    assert_eq!(q.to_string(), "let _ = [ h e l l o ]");
}
