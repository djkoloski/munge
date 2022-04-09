extern crate core;
extern crate munge;

use {
    ::core::mem::MaybeUninit,
    ::munge::munge,
};

fn main() {
    struct Example {
        a: u32,
        b: u32,
    }

    let mut mu = MaybeUninit::<Example>::uninit();

    munge!(let Example { a, b } = &mut mu);
    assert_eq!(a.write(1), &1);
    assert_eq!(b.write(2), &2);

    // SAFETY: `mu` is completely initialized.
    let value = unsafe { mu.assume_init() };
    //^ ERROR: cannot move out of `mu` because it is borrowed
    //^ NOTE: move out of `mu` occurs here
    assert_eq!(value.a, 1);
    assert_eq!(value.b, 2);

    a.write(3);
    //^ NOTE: borrow later used here
}
