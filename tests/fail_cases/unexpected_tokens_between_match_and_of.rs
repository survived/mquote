extern crate mquote;

use mquote::mquote;

fn main() {
    let _ = mquote!(
        #{match Some("1")} unexpected stuff
            #{of Some(x)}
                #{x}
            #{of None}
                "default value"
        #{endmatch}
    );
}
