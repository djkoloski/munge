use {
    crate::{Destructure, Restructure, StructuralPinning},
    ::core::{
        cell::{Cell, UnsafeCell},
        mem::{ManuallyDrop, MaybeUninit},
        pin::Pin,
    },
};

// *const T

impl<T: ?Sized> Destructure for *const T {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        *self as *mut T
    }
}

unsafe impl<T: ?Sized, U: ?Sized> Restructure<&*const T> for U {
    type Restructured = *const Self;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        ptr as *const Self
    }
}

// *mut T

impl<T: ?Sized> Destructure for *mut T {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        *self
    }
}

unsafe impl<T: ?Sized, U: ?Sized> Restructure<&*mut T> for U {
    type Restructured = *mut Self;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        ptr
    }
}

// &MaybeUninit

impl<'a, T> Destructure for &'a MaybeUninit<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.as_ptr() as *mut Self::Underlying
    }
}

unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<&'b &'a MaybeUninit<T>> for U {
    type Restructured = &'b MaybeUninit<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a MaybeUninit<T>`, so it's safe to dereference for the `'b` lifetime.
        unsafe { &*ptr.cast() }
    }
}

// &mut MaybeUninit

impl<'a, T> Destructure for &'a mut MaybeUninit<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        MaybeUninit::as_mut_ptr(self)
    }
}

unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<&'b &'a mut MaybeUninit<T>> for U {
    type Restructured = &'b mut MaybeUninit<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a mut MaybeUninit<T>`, so it's safe to mutably dereference for the `'b` lifetime.
        unsafe { &mut *ptr.cast() }
    }
}

// &Cell<T>

impl<'a, T> Destructure for &'a Cell<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<&'b &'a Cell<T>> for U {
    type Restructured = &'b Cell<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a Cell<T>`, so it's safe to dereference for the `'b` lifetime.
        unsafe { &*ptr.cast() }
    }
}

// &UnsafeCell<T>

impl<'a, T> Destructure for &'a UnsafeCell<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        self.get()
    }
}

unsafe impl<'a: 'b, 'b, T, U: 'b> Restructure<&'b &'a UnsafeCell<T>> for U {
    type Restructured = &'b UnsafeCell<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b &'a UnsafeCell<T>`, so it's safe to dereference for the `'b` lifetime.
        unsafe { &*ptr.cast() }
    }
}

// Pin<&T> where T: StructuralPinning

impl<'a, T: StructuralPinning> Destructure for Pin<&'a T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        // SAFETY: The value pointed to by `self` will continue to be treated as pinned.
        unsafe { Pin::into_inner_unchecked(self.as_ref()) as *const T as *mut T }
    }
}

unsafe impl<'a: 'b, 'b, T: StructuralPinning, U: 'b> Restructure<&'b Pin<&'a T>> for U {
    type Restructured = Pin<&'b U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b Pin<&'a T>`, and `T` has structural pinning, so it's safe to derefence as pinned for
        // the `'b` lifetime.
        unsafe { Pin::new_unchecked(&*ptr) }
    }
}

// Pin<&mut T> where T: StructuralPinning

impl<'a, T: StructuralPinning> Destructure for Pin<&'a mut T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        // SAFETY: The value pointed to by `self` will continue to be treated as pinned.
        unsafe { Pin::into_inner_unchecked(self.as_mut()) as *mut T }
    }
}

unsafe impl<'a: 'b, 'b, T: StructuralPinning, U: 'b> Restructure<&'b Pin<&'a mut T>> for U {
    type Restructured = Pin<&'b mut U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of some
        // `&'b Pin<&'a mut T>`, and `T` has structural pinning, so it's safe to mutably derefence
        // as pinned for the `'b` lifetime.
        unsafe { Pin::new_unchecked(&mut *ptr) }
    }
}

// ManuallyDrop<T>

impl<'a, T> Destructure for ManuallyDrop<T> {
    type Underlying = T;

    fn as_mut_ptr(&mut self) -> *mut Self::Underlying {
        &mut **self as *mut Self::Underlying
    }
}

unsafe impl<T, U> Restructure<&ManuallyDrop<T>> for U {
    type Restructured = ManuallyDrop<U>;

    unsafe fn restructure(ptr: *mut Self) -> Self::Restructured {
        // SAFETY: `ptr` is a pointer to a subfield of some `&ManuallyDrop<T>`.
        unsafe { ::core::ptr::read(ptr.cast()) }
    }
}
