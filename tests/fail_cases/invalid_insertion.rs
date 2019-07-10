extern crate mquote;

use mquote::mquote;

fn main() {
    let _ = mquote!(
        #{heh}
        #{requested_span}
        { #{inner_stream} }
        #{!!!}
    );
}
