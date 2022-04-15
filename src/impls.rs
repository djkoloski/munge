use {
    crate::{Destructure, Restructure, StructuralPinning},
    ::core::{
        cell::{Cell, UnsafeCell},
        mem::{ManuallyDrop, MaybeUninit},
        pin::Pin,
    },
};

// *const T

// SAFETY: Destructuring `*const T` is safe if and only if destructuring `T` with the same pattern
// is also safe.
unsafe impl<T> Destructure for *const T {
    type Underlying = T;
    type Test = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        *self as *mut T
    }
}

// SAFETY: `restructure` returns a valid `*const U` that upholds the same invariants as a mutably
// borrowed subfield of some `T`.
unsafe impl<T: ?Sized, U: ?Sized> Restructure<&*const T> for U {
    type Restructured = *const U;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        ptr as *const Self
    }
}

// *mut T

// SAFETY: Destructuring `*mut T` is safe if and only if destructuring `T` with the same pattern is
// also safe.
unsafe impl<T> Destructure for *mut T {
    type Underlying = T;
    type Test = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        *self
    }
}

// SAFETY: `restructure` returns a valid `*mut U` that upholds the same invariants as a mutably
// borrowed subfield of some `T`.
unsafe impl<T: ?Sized, U: ?Sized> Restructure<&*mut T> for U {
    type Restructured = *mut U;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        ptr
    }
}

// &MaybeUninit<T>

// SAFETY: Destructuring `&'a MaybeUninit<T>` is safe if and only if destructuring `&'a T` with the
// same pattern is also safe.
unsafe impl<'a, T> Destructure for &'a MaybeUninit<T> {
    type Underlying = T;
    type Test = &'a T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.as_ptr() as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a valid `MaybeUninit` reference that upholds the same invariants as
// a mutably borrowed subfield of some `T`.
unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<&'b &'a MaybeUninit<T>> for U {
    type Restructured = &'b MaybeUninit<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a MaybeUninit<T>`, so it's safe to dereference for the `'b` lifetime.
        unsafe { &*ptr.cast() }
    }
}

// &mut MaybeUninit<T>

// SAFETY: Destructuring `&'a mut MaybeUninit<T>` is safe if and only if destructuring `&'a mut T`
// with the same pattern is also safe.
unsafe impl<'a, T> Destructure for &'a mut MaybeUninit<T> {
    type Underlying = T;
    type Test = &'a T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        MaybeUninit::as_mut_ptr(self)
    }
}

// SAFETY: `restructure` returns a valid `MaybeUninit` reference that upholds the same invariants as
// a mutably borrowed subfield of some `T`.
unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<&'b &'a mut MaybeUninit<T>> for U {
    type Restructured = &'b mut MaybeUninit<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a mut MaybeUninit<T>`, so it's safe to mutably dereference for the `'b` lifetime.
        unsafe { &mut *ptr.cast() }
    }
}

// &Cell<T>

// SAFETY: Destructuring `&'a Cell<T>` is safe if and only if destructuring `&'a T` with the same
// pattern is also safe.
unsafe impl<'a, T: ?Sized> Destructure for &'a Cell<T> {
    type Underlying = T;
    type Test = &'a T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a valid `Cell` reference that upholds the same invariants as a
// mutably borrowed subfield of some `T`.
unsafe impl<'a: 'b, 'b, T: ?Sized, U: 'b + ?Sized> Restructure<&'b &'a Cell<T>> for U {
    type Restructured = &'b Cell<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a Cell<T>`, so it's safe to dereference for the `'b` lifetime. Additionally, `ptr`
        // is guaranteed to have the same pointer metadata as a pointer to `Cell<U>`.
        unsafe { &*::core::mem::transmute::<*mut Self, *const Cell<U>>(ptr) }
    }
}

// &UnsafeCell<T>

// SAFETY: Destructuring `&'a UnsafeCell<T>` is safe if and only if destructuring `&'a T` with the
// same pattern is also safe.
unsafe impl<'a, T: ?Sized> Destructure for &'a UnsafeCell<T> {
    type Underlying = T;
    type Test = &'a T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.get()
    }
}

// SAFETY: `restructure` returns a valid `UnsafeCell` reference that upholds the same invariants as
// a mutably borrowed subfield of some `T`.
unsafe impl<'a: 'b, 'b, T: ?Sized, U: 'b + ?Sized> Restructure<&'b &'a UnsafeCell<T>> for U {
    type Restructured = &'b UnsafeCell<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a UnsafeCell<T>`, so it's safe to dereference for the `'b` lifetime. Additionally,
        // `ptr` is guaranteed to have the same pointer metadata as a pointer to `UnsafeCell<U>`.
        unsafe { &*::core::mem::transmute::<*mut Self, *const UnsafeCell<U>>(ptr) }
    }
}

// Pin<&T> where T: StructuralPinning

// SAFETY: Destructuring `Pin<&'a T>` is safe if and only if destructuring `&'a T` with the same
// pattern is also safe.
unsafe impl<'a, T: StructuralPinning + ?Sized> Destructure for Pin<&'a T> {
    type Underlying = T;
    type Test = &'a T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        // SAFETY: The value pointed to by `self` will continue to be treated as pinned.
        unsafe { Pin::into_inner_unchecked(self.as_ref()) as *const T as *mut T }
    }
}

// SAFETY: `restructure` returns a valid `Pin<&'a T>` that upholds the same invariants as a mutably
// borrowed subfield of some `T`.
unsafe impl<'a, 'b, T, U> Restructure<&'b Pin<&'a T>> for U
where
    'a: 'b,
    T: StructuralPinning + ?Sized,
    U: 'b + ?Sized,
{
    type Restructured = Pin<&'b U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b Pin<&'a T>`, and `T` has structural pinning, so it's safe to derefence as pinned for
        // the `'b` lifetime.
        unsafe { Pin::new_unchecked(&*ptr) }
    }
}

// Pin<&mut T> where T: StructuralPinning

// SAFETY: Destructuring `Pin<&'a mut T>` is safe if and only if destructuring `&'a T` with the same
// pattern is also safe.
unsafe impl<'a, T: StructuralPinning + ?Sized> Destructure for Pin<&'a mut T> {
    type Underlying = T;
    type Test = &'a T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        // SAFETY: The value pointed to by `self` will continue to be treated as pinned.
        unsafe { Pin::into_inner_unchecked(self.as_mut()) as *mut T }
    }
}

// SAFETY: `restructure` returns a valid `Pin<&'a mut T>` that upholds the same invariants as a
// mutably borrowed subfield of some `T`.
unsafe impl<'a, 'b, T, U> Restructure<&'b Pin<&'a mut T>> for U
where
    'a: 'b,
    T: StructuralPinning + ?Sized,
    U: 'b + ?Sized,
{
    type Restructured = Pin<&'b mut U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b Pin<&'a mut T>`, and `T` has structural pinning, so it's safe to mutably derefence
        // as pinned for the `'b` lifetime.
        unsafe { Pin::new_unchecked(&mut *ptr) }
    }
}

// ManuallyDrop<T>

// SAFETY: Destructuring `ManuallyDrop<T>` is safe if and only if destructuring `T` with the same
// pattern is also safe.
unsafe impl<'a, T> Destructure for ManuallyDrop<T> {
    type Underlying = T;
    type Test = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        &mut **self as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a valid `ManuallyDrop<T>` that upholds the same invariants as a
// mutably borrowed subfield of some `T`.
unsafe impl<T, U> Restructure<&ManuallyDrop<T>> for U {
    type Restructured = ManuallyDrop<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: `ptr` is a pointer to a subfield of some `&ManuallyDrop<T>`.
        unsafe { ::core::ptr::read(ptr.cast()) }
    }
}
