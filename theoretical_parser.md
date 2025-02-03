# Theoretical parser
An abstract look at how i'm laying out the parser
```rust
add => change(2) nops add
        | change(2) add
changes(n) => change(n) | changes(n) change(n)
// important that this is 2 changes in 1 or 1 change in 2. We want
// to get the final symbol from this, and this makes it easier.
change(2) => change(1) nops change(1)
            | <pushes 2>



nops => nop | nops nop
nop => <stack no ops>
```