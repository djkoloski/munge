//! `munge` makes it easy and safe to destructure raw pointers, `MaybeUninit`s, `Cell`s, and `Pin`s.
//!
//! Just use the `munge!` macro to destructure opaque types the same way you'd destructure a value.
//!
//! `munge` has no features and is always `#![no_std]`.
//!
//! ## Examples
//!
//! `munge` makes it easy to initialize `MaybeUninit`s:
//!
//! ```rust
//! use {
//!     ::core::mem::MaybeUninit,
//!     ::munge::munge,
//! };
//!
//! pub struct Example {
//!     a: u32,
//!     b: (char, f32),
//! }
//!
//! let mut mu = MaybeUninit::<Example>::uninit();
//!
//! munge!(let Example { a, b: (c, mut f) } = &mut mu);
//! assert_eq!(a.write(10), &10);
//! assert_eq!(c.write('x'), &'x');
//! assert_eq!(f.write(3.14), &3.14);
//! // Note that `mut` bindings can be reassigned like you'd expect:
//! f = &mut MaybeUninit::uninit();
//!
//! // SAFETY: `mu` is completely initialized.
//! let init = unsafe { mu.assume_init() };
//! assert_eq!(init.a, 10);
//! assert_eq!(init.b.0, 'x');
//! assert_eq!(init.b.1, 3.14);
//! ```
//!
//! It can also be used to destructure `Cell`s:
//!
//! ```rust
//! use {
//!     ::core::cell::Cell,
//!     ::munge::munge,
//! };
//!
//! pub struct Example {
//!     a: u32,
//!     b: (char, f32),
//! }
//!
//! let value = Example {
//!     a: 10,
//!     b: ('x', 3.14),
//! };
//! let cell = Cell::<Example>::new(value);
//!
//! munge!(let Example { a, b: (c, f) } = &cell);
//! assert_eq!(a.get(), 10);
//! a.set(42);
//! assert_eq!(c.get(), 'x');
//! c.set('!');
//! assert_eq!(f.get(), 3.14);
//! f.set(1.41);
//!
//! let value = cell.into_inner();
//! assert_eq!(value.a, 42);
//! assert_eq!(value.b.0, '!');
//! assert_eq!(value.b.1, 1.41);
//! ```
//!
//! And `Pin`s as long as all fields are structurally pinned:
//!
//! ```rust
//! use {
//!     ::core::{marker::PhantomPinned, pin::Pin},
//!     ::munge::{munge, StructuralPinning},
//! };
//!
//! struct Example {
//!     pub a: u32,
//!     pub b: char,
//!     pub _phantom: PhantomPinned,
//! }
//!
//! // SAFETY: `Example` obeys structural pinning.
//! unsafe impl StructuralPinning for Example {}
//!
//! let mut value = Example {
//!     a: 0,
//!     b: ' ',
//!     _phantom: PhantomPinned,
//! };
//! // SAFETY: `value` will not be moved before being dropped.
//! let mut pin = unsafe { Pin::new_unchecked(&mut value) };
//!
//! munge!(let Example { a, b, .. } = pin.as_mut());
//! *a.get_mut() = 1;
//! *b.get_mut() = 'a';
//!
//! assert_eq!(pin.as_mut().into_ref().a, 1);
//! assert_eq!(pin.as_mut().into_ref().b, 'a');
//! assert_eq!(value.a, 1);
//! assert_eq!(value.b, 'a');
//! ```
//!
//! You can even extend `munge` to work with your own types by implementing its [`Destructure`] and
//! [`Restructure`] traits.

#![no_std]
#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

mod impls;

/// A type which has structural pinning for all of its fields.
///
/// # Safety
///
/// All fields of the implementing type must obey structural pinning. This comes with a detailed set
/// of requirements detailed in the [`pin` module documentation].
///
/// [`pin` module documentation]: ::core::pin#pinning-is-structural-for-field
pub unsafe trait StructuralPinning {}

/// A type that can be destructured into its constituent parts.
pub trait Destructure {
    /// The underlying type that is destructured.
    type Underlying: ?Sized;

    /// Returns a mutable pointer to the underlying type.
    fn as_mut_ptr(&mut self) -> *mut Self::Underlying;
}

/// A type that can be "restructured" as a field of some containing type.
///
/// # Safety
///
/// [`restructure`](Restructure::restructure) must return a valid
/// [`Restructured`](Restructure::Restructured) that upholds the same invariants as a mutably
/// borrowed subfield of some `T`. These invariants must not be violated if simultaneous mutable
/// borrows exist to other subfields of the same `T`.
pub unsafe trait Restructure<T: ?Sized> {
    /// The restructured version of this type.
    type Restructured;

    /// Restructures a pointer to this type into the target type.
    ///
    /// # Safety
    ///
    /// `ptr` must be a pointer to a subfield of some `T`.
    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured;
}

/// Projects a `MaybeUninit` type to its `MaybeUninit` fields using destructuring.
///
/// # Example
///
/// ```
/// # use { ::core::mem::MaybeUninit, ::munge::munge };
/// pub struct Example {
///     a: u32,
///     b: (char, f32),
/// }
///
/// let mut mu = MaybeUninit::<Example>::uninit();
///
/// munge!(let Example { a, b: (c, mut f) } = &mut mu);
/// assert_eq!(a.write(10), &10);
/// assert_eq!(c.write('x'), &'x');
/// assert_eq!(f.write(3.14), &3.14);
/// // Note that `mut` bindings can be reassigned like you'd expect:
/// f = &mut MaybeUninit::uninit();
///
/// // SAFETY: `mu` is completely initialized.
/// let init = unsafe { mu.assume_init() };
/// assert_eq!(init.a, 10);
/// assert_eq!(init.b.0, 'x');
/// assert_eq!(init.b.1, 3.14);
/// ```
#[macro_export]
macro_rules! munge {
    (@field($ptr:ident) $field:tt) => {{
        // SAFETY: `ptr` always is non-null, properly aligned, and valid for reads and writes.
        unsafe { ::core::ptr::addr_of_mut!((*$ptr).$field) }
    }};

    (@element($ptr:ident) $index:tt) => {{
        // SAFETY: `ptr` always is non-null, properly aligned, and valid for reads and writes.
        unsafe { ::core::ptr::addr_of_mut!((*$ptr)[$index]) }
    }};

    (@parse_binding mut $name:ident $(,)?) => {
        munge!(@bindings mut $name)
    };
    (@parse_binding mut $name:ident, $($rest:tt)+) => {(
        munge!(@bindings mut $name),
        munge!(@parse_binding $($rest)+),
    )};
    (@parse_binding $body:tt $(,)?) => {
        munge!(@bindings $body)
    };
    (@parse_binding $body:tt, $($rest:tt)+) => {(
        munge!(@bindings $body),
        munge!(@parse_binding $($rest)+),
    )};
    (@parse_binding $first:tt $($rest:tt)+) => {
        munge!(@parse_binding $($rest)+)
    };

    (@bindings _) => { _ };
    (@bindings ..) => { _ };
    (@bindings $name:ident) => { $name };
    (@bindings mut $name:ident) => { mut $name };

    (@bindings { .. }) => { _ };
    (@bindings { $field:ident $(,)? }) => { $field };
    (@bindings { mut $field:ident $(,)? }) => { mut $field };
    (@bindings { $field:ident, $($rest:tt)+ }) => { ($field, munge!(@bindings { $($rest)+ })) };
    (@bindings { mut $field:ident, $($rest:tt)+ }) => { (mut $field, munge!(@bindings { $($rest)+ })) };
    (@bindings { $field:ident: $($body:tt)+ }) => { munge!(@parse_binding $($body)+) };

    (@bindings ( $($body:tt)+ )) => { munge!(@parse_binding $($body)+) };

    (@bindings [ $($body:tt)+ ]) => { munge!(@parse_binding $($body)+) };

    (@parse_field($ptr:ident, $field:ident, $value:ident) { $body:tt $(,)? }) => {
        munge!(@fields($field, $value) $body)
    };
    (@parse_field($ptr:ident, $field:ident, $value:ident) { $body:tt, $($rest:tt)+ }) => {(
        munge!(@fields($field, $value) $body),
        munge!(@fields($ptr, $value) { $($rest)+ }),
    )};
    (@parse_field($ptr:ident, $field:ident, $value:ident) { $first:tt $($rest:tt)+ }) => {
        munge!(@parse_field($ptr, $field, $value) { $($rest)+ })
    };

    (@parse_field($ptr:ident, $index:tt, $value:ident) ( $body:tt $(,)? ) $indices:tt) => {
        munge!(@fields($index, $value) $body)
    };
    (@parse_field($ptr:ident, $index:tt, $value:ident) ( $body:tt, $($rest:tt)+ ) $indices:tt) => {(
        munge!(@fields($index, $value) $body),
        munge!(@fields($ptr, $value) ( $($rest)+ ) $indices),
    )};
    (@parse_field($ptr:ident, $index:ident, $value:ident) ( $first:tt $($rest:tt)+ ) $indices:tt) => {
        munge!(@parse_field($ptr, $index, $value) ( $($rest)+ ) $indices)
    };

    (@parse_field($ptr:ident, $index:tt, $value:ident) [ $body:tt $(,)? ] $indices:tt) => {
        munge!(@fields($index, $value) $body)
    };
    (@parse_field($ptr:ident, $index:tt, $value:ident) [ $body:tt, $($rest:tt)+ ] $indices:tt) => {(
        munge!(@fields($index, $value) $body),
        munge!(@fields($ptr, $value) [ $($rest)+ ] $indices),
    )};
    (@parse_field($ptr:ident, $index:ident, $value:ident) [ $first:tt $($rest:tt)+ ] $indices:tt) => {
        munge!(@parse_field($ptr, $index, $value) [ $($rest)+ ] $indices)
    };

    (@fields($ptr:ident, $value:ident) _) => {
        // SAFETY: This resolves to `&value` and a pointer to one of its subfields.
        unsafe { project(&$value, $ptr) }
    };
    (@fields($ptr:ident, $value:ident) $field:ident) => {
        // SAFETY: This resolves to `&value` and a pointer to one of its subfields.
        unsafe { project(&$value, $ptr) }
    };

    (@fields($ptr:ident, $value:ident) { .. }) => { () };
    (@fields($ptr:ident, $value:ident) { mut $field:ident $(,)? }) => {
        // SAFETY: This resolves to `&value` and a pointer to one of its subfields.
        unsafe { project(&$value, munge!(@field($ptr) $field)) }
    };
    (@fields($ptr:ident, $value:ident) { $field:ident $(,)? }) => {
        // SAFETY: This resolves to `&value` and a pointer to one of its subfields.
        unsafe { project(&$value, munge!(@field($ptr) $field)) }
    };
    (@fields($ptr:ident, $value:ident) { mut $field:ident, $($rest:tt)+ }) => {(
        // SAFETY: This resolves to `&value` and a pointer to one of its subfields.
        unsafe { project(&$value, munge!(@field($ptr) $field)) },
        munge!(@fields($ptr, $value) { $($rest)+ }),
    )};
    (@fields($ptr:ident, $value:ident) { $field:ident, $($rest:tt)+ }) => {(
        // SAFETY: This resolves to `&value` and a pointer to one of its subfields.
        unsafe { project(&$value, munge!(@field($ptr) $field)) },
        munge!(@fields($ptr, $value) { $($rest)+ }),
    )};
    (@fields($ptr:ident, $value:ident) { $field:ident: $($binding:tt)+ }) => {{
        let mut $field = munge!(@field($ptr) $field);
        munge!(@parse_field($ptr, $field, $value) { $($binding)+ })
    }};

    (@fields($ptr:ident, $value:ident) ( .. ) $indices:tt) => { () };
    (@fields($ptr:ident, $value:ident) ( $($binding:tt)* ) [ $index_first:tt $($index_rest:tt)* ]) => {{
        let mut index = munge!(@field($ptr) $index_first);
        munge!(@parse_field($ptr, index, $value) ( $($binding)+ ) [ $($index_rest)* ])
    }};
    (@fields($ptr:ident, $value:ident) ( $($body:tt)* )) => { munge!(@fields($ptr, $value) ( $($body)* ) [ 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 ]) };

    (@fields($ptr:ident, $value:ident) [ .. ] $indices:tt) => { () };
    (@fields($ptr:ident, $value:ident) [ $($binding:tt)* ] [ $index_first:tt $($index_rest:tt)* ]) => {{
        let mut index = munge!(@element($ptr) $index_first);
        munge!(@parse_field($ptr, index, $value) [ $($binding)+ ] [ $($index_rest)* ])
    }};
    (@fields($ptr:ident, $value:ident) [ $($body:tt)* ]) => { munge!(@fields($ptr, $value) [ $($body)* ] [ 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 ]) };

    (@fields($ptr:ident, $value:ident) $first:tt $($rest:tt)*) => { munge!(@fields($ptr, $value) $($rest)*) };

    (@destructure { $($path:tt)* } $body:tt = $value:expr) => {
        let mut value = $value;
        let munge!(@bindings $body) = {
            // Comments get thrown out when the macro gets expanded.
            #[allow(unused_mut, unused_unsafe, clippy::undocumented_unsafe_blocks)]
            {
                let ptr = $crate::Destructure::as_mut_ptr(&mut value);

                #[allow(unreachable_code, unused_variables)]
                if false {
                    // SAFETY: None of this can ever be executed.
                    unsafe {
                        ::core::hint::unreachable_unchecked();
                        let $($path)* $body = &mut ::core::ptr::read(ptr);
                    }
                }

                /// # Safety
                ///
                /// `ptr` must be a pointer to a subfield of `_value`.
                unsafe fn project<'b, T: ?Sized, U: $crate::Restructure<&'b T> + ?Sized>(
                    _value: &'b T,
                    ptr: *mut U,
                ) -> U::Restructured {
                    // SAFETY: The caller has guaranteed that `ptr` is a pointer to a subfield of
                    // `_value`, which is a `&'b T`.
                    unsafe { U::restructure(ptr) }
                }

                munge!(@fields(ptr, value) $body)
            }
        };
    };
    (@destructure { $($path:tt)* } $first:tt $($rest:tt)*) => {
        munge!(@destructure { $($path)* $first } $($rest)*)
    };

    (let $($tokens:tt)*) => {
        munge!(@destructure {} $($tokens)*)
    };
}

#[cfg(test)]
mod tests {
    use ::core::mem::MaybeUninit;

    #[test]
    fn test_project_tuple() {
        let mut mu = MaybeUninit::<(u32, char)>::uninit();

        munge!(let (a, b) = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let (a, b,) = &mut mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let (a, _) = &mut mu);
        assert_eq!(a.write(3), &3);
        munge!(let (_, b) = &mut mu);
        assert_eq!(b.write('c'), &'c');
        munge!(let (a, _,) = &mut mu);
        assert_eq!(a.write(3), &3);
        munge!(let (_, b,) = &mut mu);
        assert_eq!(b.write('c'), &'c');

        munge!(let (mut a, mut b) = &mut mu);
        assert_eq!(a.write(4), &4);
        assert_eq!(b.write('d'), &'d');
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let (a, ..) = &mut mu);
        assert_eq!(a.write(5), &5);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.0, 5);
        assert_eq!(init.1, 'd');
    }

    #[test]
    fn test_project_array() {
        let mut mu = MaybeUninit::<[u32; 2]>::uninit();

        munge!(let [a, b] = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write(1), &1);
        munge!(let [a, b,] = &mut mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write(2), &2);

        munge!(let [a, _] = &mut mu);
        assert_eq!(a.write(3), &3);
        munge!(let [_, b] = &mut mu);
        assert_eq!(b.write(3), &3);
        munge!(let [a, _,] = &mut mu);
        assert_eq!(a.write(4), &4);
        munge!(let [_, b,] = &mut mu);
        assert_eq!(b.write(4), &4);

        munge!(let [mut a, mut b] = &mut mu);
        assert_eq!(a.write(5), &5);
        assert_eq!(b.write(5), &5);
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let [a, ..] = &mut mu);
        assert_eq!(a.write(6), &6);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init[0], 6);
        assert_eq!(init[1], 5);
    }

    #[test]
    fn test_project_struct() {
        pub struct Example {
            pub a: u32,
            pub b: char,
        }

        let mut mu = MaybeUninit::<Example>::uninit();

        munge!(let Example { a, b } = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let Example { a, b, } = &mut mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let Example { a, b: x } = &mut mu);
        assert_eq!(a.write(3), &3);
        assert_eq!(x.write('c'), &'c');
        munge!(let Example { a, b: x, } = &mut mu);
        assert_eq!(a.write(4), &4);
        assert_eq!(x.write('d'), &'d');

        munge!(let Example { a: x, b } = &mut mu);
        assert_eq!(x.write(3), &3);
        assert_eq!(b.write('c'), &'c');
        munge!(let Example { a: x, b, } = &mut mu);
        assert_eq!(x.write(4), &4);
        assert_eq!(b.write('d'), &'d');

        munge!(let Example { a, b: _ } = &mut mu);
        assert_eq!(a.write(5), &5);
        munge!(let Example { a, b: _, } = &mut mu);
        assert_eq!(a.write(6), &6);

        munge!(let Example { mut a, mut b } = &mut mu);
        assert_eq!(a.write(7), &7);
        assert_eq!(b.write('e'), &'e');
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let Example { a: mut x, b: mut y } = &mut mu);
        assert_eq!(x.write(8), &8);
        assert_eq!(y.write('f'), &'f');
        x = &mut MaybeUninit::uninit();
        y = &mut MaybeUninit::uninit();
        let _ = x;
        let _ = y;

        munge!(let Example { b, .. } = &mut mu);
        assert_eq!(b.write('g'), &'g');

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.a, 8);
        assert_eq!(init.b, 'g');
    }

    #[test]
    fn test_project_tuple_struct() {
        struct Example(u32, char);

        let mut mu = MaybeUninit::<Example>::uninit();

        munge!(let Example(a, b) = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let Example(a, b,) = &mut mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let Example(a, _) = &mut mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b) = &mut mu);
        assert_eq!(b.write('c'), &'c');
        munge!(let Example(a, _,) = &mut mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b,) = &mut mu);
        assert_eq!(b.write('d'), &'d');

        munge!(let Example(mut a, mut b) = &mut mu);
        assert_eq!(a.write(4), &4);
        assert_eq!(b.write('e'), &'e');
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let Example(a, ..) = &mut mu);
        assert_eq!(a.write(5), &5);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.0, 5);
        assert_eq!(init.1, 'e');
    }

    #[test]
    fn test_project_generic() {
        struct Example<T>(u32, T);

        let mut mu = MaybeUninit::<Example<char>>::uninit();

        munge!(let Example(a, b) = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let Example(a, b,) = &mut mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let Example(a, _) = &mut mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b) = &mut mu);
        assert_eq!(b.write('c'), &'c');
        munge!(let Example(a, _,) = &mut mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b,) = &mut mu);
        assert_eq!(b.write('c'), &'c');

        munge!(let Example(a, ..) = &mut mu);
        assert_eq!(a.write(4), &4);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.0, 4);
        assert_eq!(init.1, 'c');

        let mut mu = MaybeUninit::<Example<Example<char>>>::uninit();

        munge!(let Example::<Example<char>>(a, Example::<char>(b, c)) = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write(2), &2);
        assert_eq!(c.write('a'), &'a');

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.0, 1);
        assert_eq!(init.1 .0, 2);
        assert_eq!(init.1 .1, 'a');
    }

    #[test]
    fn test_project_nested_struct() {
        struct Inner {
            a: u32,
            b: char,
        }
        struct Outer {
            inner: Inner,
            c: i32,
        }

        let mut mu = MaybeUninit::<Outer>::uninit();

        munge!(let Outer { inner: Inner { a, b }, c } = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        assert_eq!(c.write(2), &2);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.inner.a, 1);
        assert_eq!(init.inner.b, 'a');
        assert_eq!(init.c, 2);
    }

    #[test]
    fn test_project_nested_tuple() {
        let mut mu = MaybeUninit::<(u32, (char, u32))>::uninit();

        munge!(let (a, (b, c)) = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        assert_eq!(c.write(2), &2);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init, (1, ('a', 2)));
    }

    #[test]
    fn test_project_nested_array() {
        let mut mu = MaybeUninit::<[[u32; 2]; 2]>::uninit();

        munge!(let [a, [b, c]] = &mut mu);
        assert_eq!(a.write([1, 2]), &[1, 2]);
        assert_eq!(b.write(3), &3);
        assert_eq!(c.write(4), &4);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init, [[1, 2], [3, 4]]);
    }

    #[test]
    fn test_generics() {
        struct Inner<T> {
            a: u32,
            b: T,
        }
        struct Outer<T> {
            inner: Inner<T>,
            c: i32,
        }

        let mut mu = MaybeUninit::<Outer<char>>::uninit();

        munge!(let Outer { inner: Inner { a, b }, c } = &mut mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        assert_eq!(c.write(2), &2);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.inner.a, 1);
        assert_eq!(init.inner.b, 'a');
        assert_eq!(init.c, 2);
    }

    #[test]
    fn test_cell() {
        use ::core::cell::Cell;

        pub struct Example {
            a: u32,
            b: (char, f32),
        }

        let value = Example { a: 10, b: ('x', core::f32::consts::PI) };
        let cell = Cell::<Example>::new(value);

        munge!(let Example { a, b: (c, f) } = &cell);
        assert_eq!(a.get(), 10);
        a.set(42);
        assert_eq!(c.get(), 'x');
        c.set('!');
        assert_eq!(f.get(), core::f32::consts::PI);
        f.set(1.41);

        let value = cell.into_inner();
        assert_eq!(value.a, 42);
        assert_eq!(value.b.0, '!');
        assert_eq!(value.b.1, 1.41);
    }

    #[test]
    fn test_pin() {
        use {
            super::StructuralPinning,
            ::core::{marker::PhantomPinned, pin::Pin},
        };

        struct Example {
            pub a: u32,
            pub b: char,
            pub _phantom: PhantomPinned,
        }

        // SAFETY: `Example` obeys structural pinning.
        unsafe impl StructuralPinning for Example {}

        let mut value = Example { a: 0, b: ' ', _phantom: PhantomPinned };
        // SAFETY: `value` will not be moved before being dropped.
        let mut pin = unsafe { Pin::new_unchecked(&mut value) };

        munge!(let Example { a, b, .. } = pin.as_mut());
        *a.get_mut() = 1;
        *b.get_mut() = 'a';

        assert_eq!(pin.as_mut().into_ref().a, 1);
        assert_eq!(pin.as_mut().into_ref().b, 'a');
        assert_eq!(value.a, 1);
        assert_eq!(value.b, 'a');
    }

    #[test]
    fn test_manually_drop() {
        use ::core::{cell::Cell, mem::ManuallyDrop};

        struct NoisyDrop<'a> {
            counter: &'a Cell<usize>,
            value: u32,
        }

        impl<'a> Drop for NoisyDrop<'a> {
            fn drop(&mut self) {
                self.counter.set(self.counter.get() + 1);
            }
        }

        let counter = &Cell::new(0);
        assert_eq!(counter.get(), 0);

        let noisy_test = NoisyDrop { counter, value: 0 };
        drop(noisy_test);
        assert_eq!(counter.get(), 1);

        struct Example<'a> {
            a: NoisyDrop<'a>,
            b: (char, NoisyDrop<'a>),
        }

        {
            let value = Example {
                a: NoisyDrop { counter, value: 1 },
                b: ('x', NoisyDrop { counter, value: 2 }),
            };

            munge!(let Example { a, b: (c, d) } = ManuallyDrop::new(value));
            assert_eq!(a.value, 1);
            assert_eq!(*c, 'x');
            assert_eq!(d.value, 2);
        }

        assert_eq!(counter.get(), 1);
    }
}
