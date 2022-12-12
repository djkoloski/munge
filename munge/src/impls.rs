use ::core::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
};

use crate::{Destructure, Restructure};

// &MaybeUninit<T>

// SAFETY: `as_mut_ptr` returns a pointer to the `T` inside of the
// `MaybeUninit`, which must be non-null, properly aligned, and valid for reads.
unsafe impl<'a, T> Destructure for &'a MaybeUninit<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.as_ptr() as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a valid `MaybeUninit` reference that upholds
// the same invariants as a mutably borrowed subfield of some `T`.
unsafe impl<'a, T, U: 'a> Restructure<U> for &'a MaybeUninit<T> {
    type Restructured = &'a MaybeUninit<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `&'a MaybeUninit<T>`, so it's safe to dereference for the `'a`
        // lifetime.
        unsafe { &*ptr.cast() }
    }
}

// &mut MaybeUninit<T>

// SAFETY: `as_mut_ptr` returns a pointer to the `T` inside of the
// `MaybeUninit`, which must be non-null, properly aligned, and valid for reads.
unsafe impl<'a, T> Destructure for &'a mut MaybeUninit<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        MaybeUninit::as_mut_ptr(self)
    }
}

// SAFETY: `restructure` returns a valid `MaybeUninit` reference that upholds
// the same invariants as a mutably borrowed subfield of some `T`.
unsafe impl<'a, T, U: 'a> Restructure<U> for &'a mut MaybeUninit<T> {
    type Restructured = &'a mut MaybeUninit<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `&'a mut MaybeUninit<T>`, so it's safe to mutably dereference
        // for the `'a` lifetime.
        unsafe { &mut *ptr.cast() }
    }
}

// &Cell<T>

// SAFETY: `as_mut_ptr` returns a pointer to the `T` inside of the
// `Cell`, which must be non-null, properly aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for &'a Cell<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a valid `Cell` reference that upholds the same
// invariants as a mutably borrowed subfield of some `T`.
unsafe impl<'a, T: ?Sized, U: 'a + ?Sized> Restructure<U> for &'a Cell<T> {
    type Restructured = &'a Cell<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `&'a Cell<T>`, so it's safe to dereference for the `'a`
        // lifetime. Additionally, `ptr` is guaranteed to have the same pointer
        // metadata as a pointer to `Cell<U>` because `Cell` is
        // `#[repr(transparent)]`.
        unsafe { &*::core::mem::transmute::<*mut U, *const Cell<U>>(ptr) }
    }
}

// &UnsafeCell<T>

// SAFETY: `as_mut_ptr` returns a pointer to the `T` inside of the
// `UnsafeCell`, which must be non-null, properly aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for &'a UnsafeCell<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.get()
    }
}

// SAFETY: `restructure` returns a valid `UnsafeCell` reference that upholds the
// same invariants as a mutably borrowed subfield of some `T`.
unsafe impl<'a, T, U> Restructure<U> for &'a UnsafeCell<T>
where
    T: ?Sized,
    U: 'a + ?Sized,
{
    type Restructured = &'a UnsafeCell<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `&'a UnsafeCell<T>`, so it's safe to dereference for the `'a`
        // lifetime. Additionally, `ptr` is guaranteed to have the same pointer
        // metadata as a pointer to `UnsafeCell<U>`.
        unsafe { &*::core::mem::transmute::<*mut U, *const UnsafeCell<U>>(ptr) }
    }
}
