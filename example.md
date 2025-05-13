``` python
@ctx.branch(test.equals(10))
def es_gibt_keinen_guten_namen_für_diesefunktion(if_body):
    @if_body.then
    def hierauchnicht():
        pass

    @if_body.otherwise
    def hierauchnicht():
        pass
```

```rust
ctx.branch(
    test.equals(10),
    |ctx| {},
    Some(|ctx| {}),
)
```

``` rust
let fib = ctx.recursive_function(
    [Variable::from(Type::u32)]
    |this, ctx, [n]| {
        ctx.switch(
            n,
            (
                0, |ctx| 0,
                1, |ctx| 1,
                Any, |ctx| ctx.call(this, &[n - 1]) + ctx.call(thus, &[n - 2]),
            )
        )
    }
)
```
