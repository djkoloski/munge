`munge` makes it easy and safe to destructure raw pointers, `MaybeUninit`s, `Cell`s, and `Pin`s.

Just use the `munge!` macro to destructure opaque types the same way you'd destructure a value.

`munge` has no features and is always `#![no_std]`.

## Examples

`munge` makes it easy to initialize `MaybeUninit`s:

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

munge!(let Example { a, b: (c, mut f) } = &mut mu);
assert_eq!(a.write(10), &10);
assert_eq!(c.write('x'), &'x');
assert_eq!(f.write(3.14), &3.14);
// Note that `mut` bindings can be reassigned like you'd expect:
f = &mut MaybeUninit::uninit();

// SAFETY: `mu` is completely initialized.
let init = unsafe { mu.assume_init() };
assert_eq!(init.a, 10);
assert_eq!(init.b.0, 'x');
assert_eq!(init.b.1, 3.14);
```

It can also be used to destructure `Cell`s:

```rust
use {
    ::core::cell::Cell,
    ::munge::munge,
};

pub struct Example {
    a: u32,
    b: (char, f32),
}

let value = Example {
    a: 10,
    b: ('x', 3.14),
};
let cell = Cell::<Example>::new(value);

munge!(let Example { a, b: (c, f) } = &cell);
assert_eq!(a.get(), 10);
a.set(42);
assert_eq!(c.get(), 'x');
c.set('!');
assert_eq!(f.get(), 3.14);
f.set(1.41);

let value = cell.into_inner();
assert_eq!(value.a, 42);
assert_eq!(value.b.0, '!');
assert_eq!(value.b.1, 1.41);
```

And `Pin`s as long as all fields are structurally pinned:

```rust
use {
    ::core::{marker::PhantomPinned, pin::Pin},
    ::munge::{munge, StructuralPinning},
};

struct Example {
    pub a: u32,
    pub b: char,
    pub _phantom: PhantomPinned,
}

// SAFETY: `Example` obeys structural pinning.
unsafe impl StructuralPinning for Example {}

let mut value = Example {
    a: 0,
    b: ' ',
    _phantom: PhantomPinned,
};
// SAFETY: `value` will not be moved before being dropped.
let mut pin = unsafe { Pin::new_unchecked(&mut value) };

munge!(let Example { a, b, .. } = pin.as_mut());
*a.get_mut() = 1;
*b.get_mut() = 'a';

assert_eq!(pin.as_mut().into_ref().a, 1);
assert_eq!(pin.as_mut().into_ref().b, 'a');
assert_eq!(value.a, 1);
assert_eq!(value.b, 'a');
```

You can even extend `munge` to work with your own types by implementing its `Destructure` and
`Restructure` traits.
