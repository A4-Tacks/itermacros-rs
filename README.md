Some useful macros related to iterators.

# Examples
```rust
# use itermacros::iunpack;

let x = iunpack!(a, b, *c, d = 0..=5 => {
    (a, b, c, d)
} else panic!());
assert_eq!(x, (0, 1, vec![2, 3, 4], 5));
```
