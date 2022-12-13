//! `munge` makes it easy and safe to destructure `MaybeUninit`s, `Cell`s,
//! `UnsafeCell`s, `ManuallyDrop`s and more.
//!
//! Just use the `munge!` macro to destructure opaque types the same way you'd
//! destructure a value. The `munge!` macro may be used to perform both
//! reference destructuring (e.g. `let (a, b) = c` where `c` is a reference) and
//! value destructuring (e.g. `let (a, b) = c` where `c` is a value) if the
//! destructured type supports it.
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
//! You can even extend `munge` to work with your own types by implementing its
//! [`Destructure`] and [`Restructure`] traits.

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
mod internal;

#[doc(hidden)]
pub use ::munge_macro::munge_with_path;

/// Projects a type to its fields using destructuring.
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
    ($($t:tt)*) => { $crate::munge_with_path!($crate => $($t)*) }
}

/// A type that can be destructured into its constituent parts.
///
/// # Safety
///
/// - [`Destructuring`](Destructure::Destructuring) must reflect the type of
///   restructuring allowed for the type:
///   - [`Ref`] if the type may be restructured by creating disjoint borrows of
///     the fields of `Underlying`.
///   - [`Value`] if the type may be restructured by moving the fields out of
///     the destructured `Underlying`.
/// - [`underlying`](Destructure::underlying) must return a pointer that is
///   non-null, properly aligned, and valid for reads.
pub unsafe trait Destructure: Sized {
    /// The underlying type that is destructured.
    type Underlying: ?Sized;
    /// The type of destructuring to perform.
    type Destructuring;

    /// Returns a mutable pointer to the underlying type.
    fn underlying(&mut self) -> *mut Self::Underlying;
}

/// A type that can be "restructured" as a field of some containing type.
///
/// # Safety
///
/// [`restructure`](Restructure::restructure) must return a valid
/// [`Restructured`](Restructure::Restructured) that upholds the invariants for
/// its [`Destructuring`](Destructure::Destructuring):
/// - If the type is destructured [by ref](Ref), then the `Restructured` value
///   must behave as a disjoint borrow of a field of the underlying type.
/// - If the type is destructured [by value](Value), then the `Restructured`
///   value must behave as taking ownership of the fields of the underlying
///   type.
pub unsafe trait Restructure<T: ?Sized>: Destructure {
    /// The restructured version of this type.
    type Restructured;

    /// Restructures a pointer to this type into the target type.
    ///
    /// # Safety
    ///
    /// `ptr` must be a properly aligned pointer to a subfield of the pointer
    /// [`underlying`](Destructure::underlying) `self`.
    unsafe fn restructure(&self, ptr: *mut T) -> Self::Restructured;
}

/// Destructuring by reference.
///
/// e.g. `let (a, b) = c` where `c` is a reference.
pub struct Ref;

impl<T: Destructure> internal::Destructuring<T> for Ref {
    type Destructurer = internal::Ref<T>;
}

/// Destructuring by value.
///
/// e.g. `let (a, b) = c` where `c` is a value.
pub struct Value;

impl<T: Destructure> internal::Destructuring<T> for Value {
    type Destructurer = internal::Value<T>;
}

#[doc(hidden)]
pub fn make_destructurer<T: Destructure>(
    value: T,
) -> <T::Destructuring as internal::Destructuring<T>>::Destructurer
where
    T::Destructuring: internal::Destructuring<T>,
{
    internal::Destructurer::new(value)
}

#[doc(hidden)]
pub fn destructurer_ptr<T: internal::Destructurer>(
    destructurer: &mut T,
) -> *mut <T::Inner as Destructure>::Underlying {
    Destructure::underlying(destructurer.inner_mut())
}

/// # Safety
///
/// `test_destructurer` may not be called.
#[doc(hidden)]
pub fn test_destructurer<'a, T: internal::Test<'a>>(
    destructurer: &'a mut T,
) -> T::Test {
    // SAFETY: `test_destructurer` may not be called.
    unsafe { T::test(destructurer) }
}

/// # Safety
///
/// `ptr` must be a properly-aligned pointer to a subfield of the pointer
/// underlying the inner value of `destructurer`.
#[doc(hidden)]
pub unsafe fn restructure_destructurer<T: internal::Destructurer, U>(
    destructurer: &T,
    ptr: *mut U,
) -> <T::Inner as Restructure<U>>::Restructured
where
    T::Inner: Restructure<U>,
{
    // SAFETY: The caller has guaranteed that `ptr` is a properly-aligned
    // pointer to a subfield of the pointer underlying the inner value of
    // `destructurer`.
    unsafe {
        Restructure::restructure(
            internal::Destructurer::inner(destructurer),
            ptr,
        )
    }
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

        munge!(
            let Example::<Example<char>>(a, Example::<char>(b, c)) = &mut mu;
        );
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

        let value = Example {
            a: 10,
            b: ('x', core::f32::consts::PI),
        };
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
    fn test_maybe_uninit_value() {
        let mu = MaybeUninit::<(u32, char)>::new((10_000, 'x'));

        munge!(let (a, b) = mu);
        assert_eq!(unsafe { a.assume_init() }, 10_000);
        assert_eq!(unsafe { b.assume_init() }, 'x');
    }

    #[test]
    fn test_cell_value() {
        use ::core::cell::Cell;

        let cell = Cell::<(u32, char)>::new((10_000, 'x'));

        munge!(let (a, b) = cell);
        assert_eq!(a.get(), 10_000);
        assert_eq!(b.get(), 'x');
    }

    #[test]
    fn test_unsafe_cell_value() {
        use ::core::cell::UnsafeCell;

        let uc = UnsafeCell::<(u32, char)>::new((10_000, 'x'));

        munge!(let (mut a, mut b) = uc);
        assert_eq!(*a.get_mut(), 10_000);
        assert_eq!(*b.get_mut(), 'x');
    }

    #[test]
    fn test_manually_drop_value() {
        use ::core::mem::ManuallyDrop;

        let md = ManuallyDrop::new((10_000, 'x'));

        munge!(let (a, b) = md);
        assert_eq!(*a, 10_000);
        assert_eq!(*b, 'x');
    }
}
