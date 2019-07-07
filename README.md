# TODO
* \#{endfor ...}, #{endif ...}
* different code breaks
* for, match
* examples

# Motivation
The only interpolations supported by `quote!` macros are regular insertion `#a` (and
you are not able to put an expression like `my_struct.field` here) and repeating 
insertion `#(...)*`.

For me that's not enough. If you wanna conditionally put a piece of tokens, you
have to associate them with some variable and then interpolate it into `quote!`
expression:
```rust
let conditional_piece = if having_fun { 
    quote!(fn funny_method() { ... }) 
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
- [x] [Expression insertion](#expression-insertion)
- [x] [**if/else**](#if--elif--else) condition
- [x] [**for**](#for) iteration
- [ ] [**match**](#matching)ing 

This crate is not about syntax sugar only! In fact using `mquote!` in complicated
cases gives a bit of performance increasing since it does not create a several
`TokenStream`s and join them together, it handles everything within single 
`TokenStream`.

So you're able to rewrite above code:
```rust
mquote!{
    #{if having_fun}
        fn funny_method() { ... }
    #{endif}
    fn regular_method() { ... }
}
```

# More examples

## Expression insertion

```rust
fn put_filter(enabled: bool) -> TokenStream {
    let good_person = Person{ name: "Oleg", age: 20 };
    mquote!{
        assert!(!#{enabled} || person.name == #{good_person.name} 
            && person.age >= #{good_person.age})
    } 
}
```

## If / elif / else
```rust
fn define_container(amount: usize) -> TokenStream {
    mquote!{
        #{if amount > 1}
            struct People(Vec<Person>);
        #{elif amount == 1}
            struct Human(Person);
        #{else}
            struct NoneHuman;
        #{endif}
    }
}
```

## For
```rust
fn for_usage(fields: Vec<(Ident, Ident)>){
    mquote!{
        pub struct Person {
            #{for (name, ty) in fields}
                #{name}: #{ty}
            #{endfor}
        }
    }
}
```

## Matching
```rust
// TODO
```

## Escaping `#{}`
If you want to put `#{abc}` as is, you should double braces:
```rust
fn it_works() {
    let tokens = mquote!(#{{abc}});
    assert_eq!(tokens.to_string(), "#{abc}")
}
```
