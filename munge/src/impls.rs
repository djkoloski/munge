use ::core::{
    cell::{Cell, UnsafeCell},
    mem::{ManuallyDrop, MaybeUninit},
    ptr::read,
};

use crate::{Destructure, Ref, Restructure, Value};

// MaybeUninit<T>

// SAFETY:
// - `MaybeUninit<T>` is destructured by value, so its `Destructuring` type is
//   `Value`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<T> Destructure for MaybeUninit<T> {
    type Underlying = T;
    type Destructuring = Value;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr() as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a `MaybeUninit<U>` that takes ownership of the
// restructured field because `MaybeUninit<T>` is destructured by value.
unsafe impl<T, U> Restructure<U> for MaybeUninit<T> {
    type Restructured = MaybeUninit<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` is a pointer to a
        // subfield of some `T`, so it must be properly aligned, valid for
        // reads, and initialized. We may move the fields because the
        // restructuring type for `MaybeUninit<T>` is `Value`.
        unsafe { read(ptr.cast()) }
    }
}

// &MaybeUninit<T>

// SAFETY:
// - `&MaybeUninit<T>` is destructured by reference, so its `Destructuring` type
//   is `Ref`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<'a, T> Destructure for &'a MaybeUninit<T> {
    type Underlying = T;
    type Destructuring = Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr() as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a `&MaybeUninit<U>` that borrows the
// restructured field because `&MaybeUninit<T>` is destructured by reference.
unsafe impl<'a, T, U: 'a> Restructure<U> for &'a MaybeUninit<T> {
    type Restructured = &'a MaybeUninit<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `MaybeUninit<T>`, so it's safe to dereference. Because the
        // restructuring type for `&MaybeUninit<T>` is `Ref`, we may create a
        // disjoint borrow and create a reference to it for `'a`.
        unsafe { &*ptr.cast() }
    }
}

// &mut MaybeUninit<T>

// SAFETY:
// - `&mut MaybeUninit<T>` is destructured by reference, so its `Destructuring`
//   type is `Ref`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<'a, T> Destructure for &'a mut MaybeUninit<T> {
    type Underlying = T;
    type Destructuring = Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        MaybeUninit::as_mut_ptr(self)
    }
}

// SAFETY: `restructure` returns a `&mut MaybeUninit<U>` that borrows the
// restructured field because `&mut MaybeUninit<T>` is destructured by
// reference.
unsafe impl<'a, T, U: 'a> Restructure<U> for &'a mut MaybeUninit<T> {
    type Restructured = &'a mut MaybeUninit<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `MaybeUninit<T>`, so it's safe to dereference. Because the
        // restructuring type for `&mut MaybeUninit<T>` is `Ref`, we may create
        // a disjoint borrow and create a reference to it for `'a`.
        unsafe { &mut *ptr.cast() }
    }
}

// Cell<T>

// SAFETY:
// - `Cell<T>` is destructured by value, so its `Destructuring` type is `Value`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<T> Destructure for Cell<T> {
    type Underlying = T;
    type Destructuring = Value;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a `Cell<U>` that takes ownership of the
// restructured field because `Cell<T>` is destructured by value.
unsafe impl<T, U> Restructure<U> for Cell<T> {
    type Restructured = Cell<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        let ptr =
            // SAFETY: `Cell<U>` is `repr(transparent)` and so guaranteed to
            // have the same representation as the `U` it contains. Therefore,
            // the pointer metadata for `*const Cell<U>` is the same as the
            // metadata for `*mut U`, and transmuting between the two types is
            // sound.
            unsafe { ::core::mem::transmute::<*mut U, *const Cell<U>>(ptr) };
        // SAFETY: The caller has guaranteed that `ptr` is a pointer to a
        // subfield of some `T`, so it must be properly aligned, valid for
        // reads, and initialized. We may move the fields because the
        // restructuring type for `Cell<T>` is `Value`.
        unsafe { read(ptr) }
    }
}

// &Cell<T>

// SAFETY:
// - `&Cell<T>` is destructured by reference, so its `Destructuring` type is
//   `Ref`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for &'a Cell<T> {
    type Underlying = T;
    type Destructuring = Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a `&Cell<U>` that borrows the restructured
// field because `&Cell<T>` is destructured by reference.
unsafe impl<'a, T: ?Sized, U: 'a + ?Sized> Restructure<U> for &'a Cell<T> {
    type Restructured = &'a Cell<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        let ptr =
            // SAFETY: `Cell<U>` is `repr(transparent)` and so guaranteed to
            // have the same representation as the `U` it contains. Therefore,
            // the pointer metadata for `*const Cell<U>` is the same as the
            // metadata for `*mut U`, and transmuting between the two types is
            // sound.
            unsafe { ::core::mem::transmute::<*mut U, *const Cell<U>>(ptr) };
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `Cell<T>`, so it's safe to dereference. Because the
        // restructuring type for `&Cell<T>` is `Ref`, we may create a disjoint
        // borrow and create a reference to it for `'a`.
        unsafe { &*ptr }
    }
}

// &mut Cell<T>

// SAFETY:
// - `&mut Cell<T>` is destructured by reference, so its `Destructuring` type is
//   `Ref`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for &'a mut Cell<T> {
    type Underlying = T;
    type Destructuring = Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a `&mut Cell<U>` that borrows the restructured
// field because `&mut Cell<T>` is destructured by reference.
unsafe impl<'a, T: ?Sized, U: 'a + ?Sized> Restructure<U> for &'a mut Cell<T> {
    type Restructured = &'a mut Cell<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        let ptr =
            // SAFETY: `Cell<U>` is `repr(transparent)` and so guaranteed to
            // have the same representation as the `U` it contains. Therefore,
            // the pointer metadata for `*mut Cell<U>` is the same as the
            // metadata for `*mut U`, and transmuting between the two types is
            // sound.
            unsafe { ::core::mem::transmute::<*mut U, *mut Cell<U>>(ptr) };
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `Cell<T>`, so it's safe to dereference. Because the
        // restructuring type for `&mut Cell<T>` is `Ref`, we may create a
        // disjoint borrow and create a reference to it for `'a`.
        unsafe { &mut *ptr }
    }
}

// UnsafeCell<T>

// SAFETY:
// - `UnsafeCell<T>` is destructured by value, so its `Destructuring` type is
//   `Value`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<T> Destructure for UnsafeCell<T> {
    type Underlying = T;
    type Destructuring = Value;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.get()
    }
}

// SAFETY: `restructure` returns a `UnsafeCell<U>` that takes ownership of the
// restructured field because `UnsafeCell<T>` is destructured by value.
unsafe impl<T, U> Restructure<U> for UnsafeCell<T> {
    type Restructured = UnsafeCell<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: `UnsafeCell<U>` is `repr(transparent)` and so guaranteed to
        // have the same representation as the `U` it contains. Therefore, the
        // pointer metadata for `*const UnsafeCell<U>` is the same as the
        // metadata for `*mut U`, and transmuting between the two types is
        // sound.
        let ptr = unsafe {
            ::core::mem::transmute::<*mut U, *const UnsafeCell<U>>(ptr)
        };
        // SAFETY: The caller has guaranteed that `ptr` is a pointer to a
        // subfield of some `T`, so it must be properly aligned, valid for
        // reads, and initialized. We may move the fields because the
        // restructuring type for `UnsafeCell<T>` is `Value`.
        unsafe { read(ptr) }
    }
}

// &mut UnsafeCell<T>

// SAFETY:
// - `&mut UnsafeCell<T>` is destructured by reference, so its `Destructuring`
//   type is `Ref`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for &'a mut UnsafeCell<T> {
    type Underlying = T;
    type Destructuring = Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.get_mut()
    }
}

// SAFETY: `restructure` returns a `&mut UnsafeCell<U>` that borrows the
// restructured field because `&mut UnsafeCell<T>` is destructured by reference.
unsafe impl<'a, T, U> Restructure<U> for &'a mut UnsafeCell<T>
where
    T: ?Sized,
    U: 'a + ?Sized,
{
    type Restructured = &'a mut UnsafeCell<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: `UnsafeCell<U>` is `repr(transparent)` and so guaranteed to
        // have the same representation as the `U` it contains. Therefore, the
        // pointer metadata for `*mut UnsafeCell<U>` is the same as the metadata
        // for `*mut U`, and transmuting between the two types is sound.
        let ptr = unsafe {
            ::core::mem::transmute::<*mut U, *mut UnsafeCell<U>>(ptr)
        };
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `UnsafeCell<T>`, so it's safe to dereference. Because the
        // restructuring type for `&mut UnsafeCell<T>` is `Ref`, we may create a
        // disjoint borrow and create a reference to it for `'a`.
        unsafe { &mut *ptr }
    }
}

// ManuallyDrop<T>

// SAFETY:
// - `ManuallyDrop<T>` is destructured by value, so its `Destructuring` type is
//   `Value`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<T> Destructure for ManuallyDrop<T> {
    type Underlying = T;
    type Destructuring = Value;

    fn underlying(&mut self) -> *mut Self::Underlying {
        &mut **self as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a `ManuallyDrop<U>` that takes ownership of the
// restructured field because `ManuallyDrop<T>` is destructured by value.
unsafe impl<T, U> Restructure<U> for ManuallyDrop<T> {
    type Restructured = ManuallyDrop<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` is a pointer to a
        // subfield of some `T`, so it must be properly aligned, valid for
        // reads, and initialized. We may move the fields because the
        // restructuring type for `ManuallyDrop<T>` is `Value`.
        unsafe { read(ptr.cast()) }
    }
}

// &ManuallyDrop<T>

// SAFETY:
// - `&ManuallyDrop<T>` is destructured by reference, so its `Destructuring`
//   type is `Ref`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<'a, T> Destructure for &'a ManuallyDrop<T> {
    type Underlying = T;
    type Destructuring = Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        (&***self as *const Self::Underlying).cast_mut()
    }
}

// SAFETY: `restructure` returns a `&ManuallyDrop<U>` that borrows the
// restructured field because `&ManuallyDrop<T>` is destructured by reference.
unsafe impl<'a, T, U: 'a> Restructure<U> for &'a ManuallyDrop<T> {
    type Restructured = &'a ManuallyDrop<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `ManuallyDrop<T>`, so it's safe to dereference. Because the
        // restructuring type for `&ManuallyDrop<T>` is `Ref`, we may create a
        // disjoint borrow and create a reference to it for `'a`.
        unsafe { &*ptr.cast() }
    }
}

// &mut ManuallyDrop<T>

// SAFETY:
// - `&mut ManuallyDrop<T>` is destructured by reference, so its `Destructuring`
//   type is `Ref`.
// - `underlying` returns a pointer to its inner type, so it is guaranteed to be
//   non-null, properly aligned, and valid for reads.
unsafe impl<'a, T> Destructure for &'a mut ManuallyDrop<T> {
    type Underlying = T;
    type Destructuring = Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        &mut ***self as *mut Self::Underlying
    }
}

// SAFETY: `restructure` returns a `&mut ManuallyDrop<U>` that borrows the
// restructured field because `&mut ManuallyDrop<T>` is destructured by
// reference.
unsafe impl<'a, T, U: 'a> Restructure<U> for &'a mut ManuallyDrop<T> {
    type Restructured = &'a mut ManuallyDrop<U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: The caller has guaranteed that `ptr` points to a subfield of
        // some `ManuallyDrop<T>`, so it's safe to dereference. Because the
        // restructuring type for `&mut ManuallyDrop<T>` is `Ref`, we may create
        // a disjoint borrow and create a reference to it for `'a`.
        unsafe { &mut *ptr.cast() }
    }
}
