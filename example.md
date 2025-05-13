``` python
@If(test.equals(10))
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
    n.equals(10),
    |ctx| {},
    Some(|ctx| {}),
)
```

``` rust
let fib = ctx.recursive_function(
    [Variable("n".to_string(), Type::u32)]
    |this, ctx, [n]| {
        ctx.switch(
            n,
            (
                Pattern::from(0), |ctx| Value::from(0),
                Pattern::from(1), |ctx| Value::from(1),
                Pattern::Any, |ctx| ctx.call(this, &[n - 1]) + ctx.call(thus, &[n - 2]),
            )
        )
    }
)

```
