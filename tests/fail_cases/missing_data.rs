extern crate mquote;

use mquote::mquote;

fn main() {
    let _ = mquote!(#{if} 123 #{endif});
    let _ = mquote!(#{if false} 1 #{elif true} 2 #{else} 3 #{endif});
    let _ = mquote!(#{for} repeative #{endfor});
    let _ = mquote!(#{match}#{of Some(x)} #{x} #{endmatch});
    let _ = mquote!(#{match Some("1")} #{of} hi #{endmatch});
}
