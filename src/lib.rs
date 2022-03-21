//! `munge` makes it easy to initialize `MaybeUninit`s.
//!
//! Just use the `munge!` macro to destructure `MaybeUninit`s the same way you'd destructure a
//! value. Initialize all the fields, then call `assume_init` to unwrap it.
//!
//! `munge` has no features and is always `#![no_std]`.
//!
//! ## Example
//!
//! ```
//! # use { ::core::mem::MaybeUninit, ::munge::munge };
//! pub struct Example {
//!     a: u32,
//!     b: (char, f32),
//! }
//!
//! let mut mu = MaybeUninit::<Example>::uninit();
//!
//! munge!(let Example { a, b: (c, mut f) } = mu);
//! assert_eq!(a.write(10), &10);
//! assert_eq!(c.write('x'), &'x');
//! assert_eq!(f.write(3.14), &3.14);
//! f = &mut MaybeUninit::uninit();
//!
//! // SAFETY: `mu` is completely initialized.
//! let init = unsafe { mu.assume_init() };
//! assert_eq!(init.a, 10);
//! assert_eq!(init.b.0, 'x');
//! assert_eq!(init.b.1, 3.14);
//! ```

#![no_std]
#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

use ::core::{marker::PhantomData, mem::MaybeUninit};

/// A memory location that may or may not be initialized.
pub struct Munge<'a, T> {
    ptr: *mut T,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Munge<'a, T> {
    /// Create a new `Munge` from a backing `MaybeUninit<T>`.
    pub fn new(value: &'a mut MaybeUninit<T>) -> Self {
        Self {
            ptr: value.as_mut_ptr(),
            _phantom: PhantomData,
        }
    }

    /// Create a new `Munge` from an exclusive pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null, properly aligned, and valid for reads and writes.
    /// - `ptr` must not alias any other accessible references.
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Reborrows this `Munge`.
    ///
    /// This method is useful when doing multiple calls to functions that consume the `Munge`.
    pub fn as_mut<'b>(&mut self) -> Munge<'b, T>
    where
        'a: 'b,
    {
        Self {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }

    /// Gets a pointer to the underlying memory.
    pub fn as_ptr(self) -> *mut T {
        self.ptr
    }

    /// Returns a reference to the underlying `MaybeUninit`.
    pub fn deref(self) -> &'a mut MaybeUninit<T> {
        // SAFETY: `self.ptr` is always a valid pointer to a `MaybeUninit<T>`.
        unsafe { &mut *self.ptr.cast() }
    }
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
/// munge!(let Example { a, b: (c, mut f) } = mu);
/// assert_eq!(a.write(10), &10);
/// assert_eq!(c.write('x'), &'x');
/// assert_eq!(f.write(3.14), &3.14);
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
    (@field($value:ident) $field:tt) => {{
        let munge = $crate::Munge::as_mut(&mut $value);
        let ptr = $crate::Munge::as_ptr(munge);
        // SAFETY: `ptr` always is non-null, properly aligned, and valid for reads and writes.
        let field = unsafe { ::core::ptr::addr_of_mut!((*ptr).$field) };
        // SAFETY: `field` is a subfield of `value`.
        unsafe { project_lifetime(&mut $value, field) }
    }};

    (@element($value:ident) $index:tt) => {{
        let munge = $crate::Munge::as_mut(&mut $value);
        let ptr = $crate::Munge::as_ptr(munge);
        // SAFETY: `ptr` always is non-null, properly aligned, and valid for reads and writes.
        let element = unsafe { ::core::ptr::addr_of_mut!((*ptr)[$index]) };
        // SAFETY: `element` is an element of `value`.
        unsafe { project_lifetime(&mut $value, element) }
    }};

    (@helper($value:ident) $($tokens:tt)+) => {
        /// Helper function to assign the correct lifetime to a [`Munge`] when projecting to a
        /// field.
        ///
        /// # Safety
        ///
        /// `ptr` must be a pointer to a subsection of the given `Munge`.
        #[inline(always)]
        unsafe fn project_lifetime<'a: 'b, 'b, T, U>(
            _: &mut $crate::Munge<'a, U>,
            ptr: *mut T,
        ) -> $crate::Munge<'b, T> {
            // SAFETY:
            // - `ptr` is non-null, properly aligned, and valid for reads and writes.
            // - `ptr` does not alias any accessible references.
            // - `ptr` is structurally pinned.
            unsafe { $crate::Munge::new_unchecked(ptr) }
        }

        #[allow(unreachable_code, unused_variables)]
        if false {
            // SAFETY: None of this can ever be executed.
            unsafe {
                ::core::hint::unreachable_unchecked();
                let inner_ref = $value.deref();
                let $($tokens)+ = ::core::ptr::read(inner_ref).assume_init();
            }
        }
    };

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

    (@parse_field($value:ident, $field:ident) { $body:tt $(,)? }) => {
        munge!(@fields($field) $body)
    };
    (@parse_field($value:ident, $field:ident) { $body:tt, $($rest:tt)+ }) => {(
        munge!(@fields($field) $body),
        munge!(@fields($value) { $($rest)+ }),
    )};
    (@parse_field($value:ident, $field:ident) { $first:tt $($rest:tt)+ }) => {
        munge!(@parse_field($value, $field) { $($rest)+ })
    };

    (@parse_field($value:ident, $index:tt) ( $body:tt $(,)? ) $indices:tt) => {
        munge!(@fields($index) $body)
    };
    (@parse_field($value:ident, $index:tt) ( $body:tt, $($rest:tt)+ ) $indices:tt) => {(
        munge!(@fields($index) $body),
        munge!(@fields($value) ( $($rest)+ ) $indices),
    )};
    (@parse_field($value:ident, $index:ident) ( $first:tt $($rest:tt)+ ) $indices:tt) => {
        munge!(@parse_field($value, $index) ( $($rest)+ ) $indices)
    };

    (@parse_field($value:ident, $index:tt) [ $body:tt $(,)? ] $indices:tt) => {
        munge!(@fields($index) $body)
    };
    (@parse_field($value:ident, $index:tt) [ $body:tt, $($rest:tt)+ ] $indices:tt) => {(
        munge!(@fields($index) $body),
        munge!(@fields($value) [ $($rest)+ ] $indices),
    )};
    (@parse_field($value:ident, $index:ident) [ $first:tt $($rest:tt)+ ] $indices:tt) => {
        munge!(@parse_field($value, $index) [ $($rest)+ ] $indices)
    };

    (@fields($value:ident) _) => { unsafe { $crate::Munge::deref($value) } };
    (@fields($value:ident) $field:ident) => { unsafe { $crate::Munge::deref($value) } };

    (@fields($value:ident) { .. }) => { () };
    (@fields($value:ident) { mut $field:ident $(,)? }) => {
        unsafe { $crate::Munge::deref(munge!(@field($value) $field)) }
    };
    (@fields($value:ident) { $field:ident $(,)? }) => {
        unsafe { $crate::Munge::deref(munge!(@field($value) $field)) }
    };
    (@fields($value:ident) { mut $field:ident, $($rest:tt)+ }) => {(
        unsafe { $crate::Munge::deref(munge!(@field($value) $field)) },
        munge!(@fields($value) { $($rest)+ }),
    )};
    (@fields($value:ident) { $field:ident, $($rest:tt)+ }) => {(
        unsafe { $crate::Munge::deref(munge!(@field($value) $field)) },
        munge!(@fields($value) { $($rest)+ }),
    )};
    (@fields($value:ident) { $field:ident: $($binding:tt)+ }) => {{
        let mut $field = munge!(@field($value) $field);
        munge!(@parse_field($value, $field) { $($binding)+ })
    }};

    (@fields($value:ident) ( .. ) $indices:tt) => { () };
    (@fields($value:ident) ( $($binding:tt)* ) [ $index_first:tt $($index_rest:tt)* ]) => {{
        let mut index = munge!(@field($value) $index_first);
        munge!(@parse_field($value, index) ( $($binding)+ ) [ $($index_rest)* ])
    }};
    (@fields($value:ident) ( $($body:tt)* )) => { munge!(@fields($value) ( $($body)* ) [ 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 ]) };

    (@fields($value:ident) [ .. ] $indices:tt) => { () };
    (@fields($value:ident) [ $($binding:tt)* ] [ $index_first:tt $($index_rest:tt)* ]) => {{
        let mut index = munge!(@element($value) $index_first);
        munge!(@parse_field($value, index) [ $($binding)+ ] [ $($index_rest)* ])
    }};
    (@fields($value:ident) [ $($body:tt)* ]) => { munge!(@fields($value) [ $($body)* ] [ 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 ]) };

    (@fields($value:ident) $first:tt $($rest:tt)*) => { munge!(@fields($value) $($rest)*) };

    (@destructure { $($path:tt)* } $body:tt = $mu:expr) => {
        // Comments get thrown out when the macro gets expanded.
        let munge!(@bindings $body) = {
            #[allow(unused_mut, unused_unsafe, clippy::undocumented_unsafe_blocks)]
            {
                let mut value = $crate::Munge::new(&mut $mu);

                munge!(@helper(value) $($path)* $body);
                munge!(@fields(value) $body)
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

        munge!(let (a, b) = mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let (a, b,) = mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let (a, _) = mu);
        assert_eq!(a.write(3), &3);
        munge!(let (_, b) = mu);
        assert_eq!(b.write('c'), &'c');
        munge!(let (a, _,) = mu);
        assert_eq!(a.write(3), &3);
        munge!(let (_, b,) = mu);
        assert_eq!(b.write('c'), &'c');

        munge!(let (mut a, mut b) = mu);
        assert_eq!(a.write(4), &4);
        assert_eq!(b.write('d'), &'d');
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let (a, ..) = mu);
        assert_eq!(a.write(5), &5);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.0, 5);
        assert_eq!(init.1, 'd');
    }

    #[test]
    fn test_project_array() {
        let mut mu = MaybeUninit::<[u32; 2]>::uninit();

        munge!(let [a, b] = mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write(1), &1);
        munge!(let [a, b,] = mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write(2), &2);

        munge!(let [a, _] = mu);
        assert_eq!(a.write(3), &3);
        munge!(let [_, b] = mu);
        assert_eq!(b.write(3), &3);
        munge!(let [a, _,] = mu);
        assert_eq!(a.write(4), &4);
        munge!(let [_, b,] = mu);
        assert_eq!(b.write(4), &4);

        munge!(let [mut a, mut b] = mu);
        assert_eq!(a.write(5), &5);
        assert_eq!(b.write(5), &5);
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let [a, ..] = mu);
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

        munge!(let Example { a, b } = mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let Example { a, b, } = mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let Example { a, b: x } = mu);
        assert_eq!(a.write(3), &3);
        assert_eq!(x.write('c'), &'c');
        munge!(let Example { a, b: x, } = mu);
        assert_eq!(a.write(4), &4);
        assert_eq!(x.write('d'), &'d');

        munge!(let Example { a: x, b } = mu);
        assert_eq!(x.write(3), &3);
        assert_eq!(b.write('c'), &'c');
        munge!(let Example { a: x, b, } = mu);
        assert_eq!(x.write(4), &4);
        assert_eq!(b.write('d'), &'d');

        munge!(let Example { a, b: _ } = mu);
        assert_eq!(a.write(5), &5);
        munge!(let Example { a, b: _, } = mu);
        assert_eq!(a.write(6), &6);

        munge!(let Example { mut a, mut b } = mu);
        assert_eq!(a.write(7), &7);
        assert_eq!(b.write('e'), &'e');
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let Example { a: mut x, b: mut y } = mu);
        assert_eq!(x.write(8), &8);
        assert_eq!(y.write('f'), &'f');
        x = &mut MaybeUninit::uninit();
        y = &mut MaybeUninit::uninit();
        let _ = x;
        let _ = y;

        munge!(let Example { b, .. } = mu);
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

        munge!(let Example(a, b) = mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let Example(a, b,) = mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let Example(a, _) = mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b) = mu);
        assert_eq!(b.write('c'), &'c');
        munge!(let Example(a, _,) = mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b,) = mu);
        assert_eq!(b.write('d'), &'d');

        munge!(let Example(mut a, mut b) = mu);
        assert_eq!(a.write(4), &4);
        assert_eq!(b.write('e'), &'e');
        a = &mut MaybeUninit::uninit();
        b = &mut MaybeUninit::uninit();
        let _ = a;
        let _ = b;

        munge!(let Example(a, ..) = mu);
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

        munge!(let Example(a, b) = mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        munge!(let Example(a, b,) = mu);
        assert_eq!(a.write(2), &2);
        assert_eq!(b.write('b'), &'b');

        munge!(let Example(a, _) = mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b) = mu);
        assert_eq!(b.write('c'), &'c');
        munge!(let Example(a, _,) = mu);
        assert_eq!(a.write(3), &3);
        munge!(let Example(_, b,) = mu);
        assert_eq!(b.write('c'), &'c');

        munge!(let Example(a, ..) = mu);
        assert_eq!(a.write(4), &4);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.0, 4);
        assert_eq!(init.1, 'c');

        let mut mu = MaybeUninit::<Example<Example<char>>>::uninit();

        munge!(let Example::<Example<char>>(a, Example::<char>(b, c)) = mu);
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

        munge!(let Outer { inner: Inner { a, b }, c } = mu);
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

        munge!(let (a, (b, c)) = mu);
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

        munge!(let [a, [b, c]] = mu);
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

        munge!(let Outer { inner: Inner { a, b }, c } = mu);
        assert_eq!(a.write(1), &1);
        assert_eq!(b.write('a'), &'a');
        assert_eq!(c.write(2), &2);

        // SAFETY: `mu` is completely initialized.
        let init = unsafe { mu.assume_init() };
        assert_eq!(init.inner.a, 1);
        assert_eq!(init.inner.b, 'a');
        assert_eq!(init.c, 2);
    }
}
