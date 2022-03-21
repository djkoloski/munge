`munge` makes it easy to initialize `MaybeUninit`s.

Just use the `munge!` macro to destructure `MaybeUninit`s the same way you'd destructure a value.
Initialize all the fields, then call `assume_init` to unwrap it.

`munge` has no features and is always `#![no_std]`.

## Example

```rust
use {
    ::core::mem::MaybeUninit,
    ::munge::munge,
};

pub struct Example {
    a: u32,
    b: (char, f32),
}

let mut mu = MaybeUninit::<Example>::uninit();

munge!(let Example { a, b: (c, mut f) } = mu);
assert_eq!(a.write(10), &10);
assert_eq!(c.write('x'), &'x');
assert_eq!(f.write(3.14), &3.14);
f = &mut MaybeUninit::uninit();

// SAFETY: `mu` is completely initialized.
let init = unsafe { mu.assume_init() };
assert_eq!(init.a, 10);
assert_eq!(init.b.0, 'x');
assert_eq!(init.b.1, 3.14);
```
