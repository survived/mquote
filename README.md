# TODO
* remove dependency on quote and proc_macro2 from consumer
* parsing
* for, match
* examples

# Motivation
The only interpolations supported by `quote!` macros are regular insertion `#a` (and
you are not able to put an expression like `my_struct.field` here) and repeating 
insertion (`#(...)*`).

For me that's not enough. If you wanna conditionally put a piece of tokens, you
have to associate them with some variable and then interpolate it into `quote!`
expression:
```rust
let conditional_piece = if having_fun { 
    quote!(fn funny_method() -> i32 { 42 }) 
} else { 
    quote!() 
};
quote!(
    #conditional_piece
    fn regular_method() { ... }
)
```

Don't you find it could be confusing? Especially if there're a lot of such things.
Even putting simple expression like `my_struct.field` must be handled in this way.

# Introduce templating `mquote!`
It supports:
- [x] Expression insertion
- [x] **if/else** condition
- [ ] **for** iteration
- [ ] **match**ing 

This crate is not about syntax sugar only! In fact using `mquote!` in complicated
cases gives a bit of performance increasing since it does not create a several
`TokenStream`s and join them together, it handles everything within single 
`TokenStream`.

# Examples

```rust
// TODO: incorrect example
fn for_usage(){
    let fields = vec![("name", false, "String"), ("age", true, "u8")];
    mquote!{
        pub struct Person {
            #{for field in fields}
                #{field.0}: #{field.1}
            #{endfor}
        }
    }
}
```
