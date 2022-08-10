use {
    crate::{Destructure, Restructure},
    ::core::{
        cell::{Cell, UnsafeCell},
        mem::{ManuallyDrop, MaybeUninit},
    },
};

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
unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<U> for &'b &'a MaybeUninit<T> {
    type Restructured = &'b MaybeUninit<U>;

    unsafe fn restructure(ptr: *mut U) -> Self::Restructured {
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
unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<U> for &'b &'a mut MaybeUninit<T> {
    type Restructured = &'b mut MaybeUninit<U>;

    unsafe fn restructure(ptr: *mut U) -> Self::Restructured {
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
unsafe impl<'a: 'b, 'b, T: ?Sized, U: 'b + ?Sized> Restructure<U> for &'b &'a Cell<T> {
    type Restructured = &'b Cell<U>;

    unsafe fn restructure(ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a Cell<T>`, so it's safe to dereference for the `'b` lifetime. Additionally, `ptr`
        // is guaranteed to have the same pointer metadata as a pointer to `Cell<U>`.
        unsafe { &*::core::mem::transmute::<*mut U, *const Cell<U>>(ptr) }
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
unsafe impl<'a: 'b, 'b, T: ?Sized, U: 'b + ?Sized> Restructure<U> for &'b &'a UnsafeCell<T> {
    type Restructured = &'b UnsafeCell<U>;

    unsafe fn restructure(ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a UnsafeCell<T>`, so it's safe to dereference for the `'b` lifetime. Additionally,
        // `ptr` is guaranteed to have the same pointer metadata as a pointer to `UnsafeCell<U>`.
        unsafe { &*::core::mem::transmute::<*mut U, *const UnsafeCell<U>>(ptr) }
    }
}

// ManuallyDrop<T>

// SAFETY: Destructuring `ManuallyDrop<T>` is safe if and only if destructuring `T` with the same
// pattern is also safe.
unsafe impl<T> Destructure for ManuallyDrop<T> {
    type Underlying = T;
    type Test = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        &mut **self as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a valid `ManuallyDrop<T>` that upholds the same invariants as a
// mutably borrowed subfield of some `T`.
unsafe impl<T, U> Restructure<U> for &ManuallyDrop<T> {
    type Restructured = ManuallyDrop<U>;

    unsafe fn restructure(ptr: *mut U) -> Self::Restructured {
        // SAFETY: `ptr` is a pointer to a subfield of some `&ManuallyDrop<T>`.
        unsafe { ::core::ptr::read(ptr.cast()) }
    }
}
