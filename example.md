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
    |ctx| {},
    Some(|ctx| {}),
)
```
