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
- [x] [**match**](#matching)ing 
- [x] [**Extend**]ing

So you're able to rewrite above code:
```rust
mquote!{
    #{if having_fun}
        fn funny_method() { ... }
    #{endif}
    fn regular_method() { ... }
}
```

This crate is not about syntax sugar only! In fact using `mquote!` in complicated
cases gives a bit of performance increasing since it does not create a several
`TokenStream`s and join them together, it handles everything within single 
`TokenStream`.

# More examples

## Expression insertion

```rust
fn put_filter(enabled: bool) ->  proc_quote2::TokenStream {
    let good_person = Person{ name: "Oleg", age: 20 };
    mquote!{
        assert!(!#{enabled} || person.name == #{good_person.name} 
            && person.age >= #{good_person.age})
    } 
}
```

## If / elif / else
```rust
fn define_container(amount: usize) ->  proc_quote2::TokenStream {
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
fn define_person(fields: Vec<(Ident, Ident)>) -> proc_quote2::TokenStream {
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
fn hardcode_it(var: Ident, value: Option<&str>) -> proc_quote2::TokenStream {
    mquote!{
        static #var: &str = #{match value}
            #{of Some(x) if x.len() > 0}
                #{x};
            #{of Some(_)}
                "case for empty strings";
            #{of None}
                "default value";
        #{endmatch}
    }
}
```

## Extending
Sometimes you want `mquote!` to consume an iterator of `TokenTree`s
without cloning. It's possible with special syntax `^{iterable}` that accepts
any `IntoIterator<Item=TokenTree>`.

```rust
fn assign_by_ref(stream: TokenStream) -> TokenStream {
    let tail = stream.into_iter().drop(5); // here could be something
                                           // more reasonable
    mquote!{
        let _ = ^{tail}
    }
}
```

## Escaping `#{}` or `^{}`
If you want to put either `#{abc}` or `^{abc}` as is, you should double braces:
```rust
fn it_works() {
    let tokens = mquote!(#{{abc}} ^{{abc}});
    assert_eq!(tokens.to_string(), "# { abc } ^ { abc }")
}
```
