use ::deep_maybe_uninit::{DeepMaybeUninit, HasDeepMaybeUninit, IsDeepMaybeUninit};

#[derive(Debug, DeepMaybeUninit)]
#[repr(C)]
struct SimpleTest {
    a: u32,
    b: u64,
    c: char,
}

#[derive(Debug, DeepMaybeUninit)]
#[repr(C)]
struct NestedTest {
    nested: SimpleTest,
    structs: SimpleTest,
}

#[derive(Debug, DeepMaybeUninit)]
#[repr(C)]
struct ZSTTest;

#[derive(Debug, DeepMaybeUninit)]
#[repr(C)]
struct TupleTest(&'static str);

#[test]
fn simpe_test() {
    let mut x = SimpleTest::uninit();
    x.a.write(1);
    x.b.write(2);
    x.c.write('3');
    let x = unsafe { x.assume_init() };
    dbg!(x);
}

#[test]
fn nested_test() {
    let mut x = NestedTest::uninit();
    let nested = &mut x.nested;
    let structs = &mut x.structs;
    nested.a.write(1);
    nested.b.write(2);
    nested.c.write('3');
    structs.a.write(4);
    structs.b.write(5);
    structs.c.write('6');
    let x = unsafe { x.assume_init() };
    // structs.c.write('7'); // moved out of
    dbg!(x);
}

#[test]
fn boxed_nested_test() {
    let mut x: Box<_> = NestedTest::boxed_uninit();
    x.nested.a.write(1);
    x.nested.b.write(2);
    x.nested.c.write('3');
    x.structs.a.write(4);
    x.structs.b.write(5);
    x.structs.c.write('6');
    let x = unsafe { x.boxed_assume_init() };
    dbg!(x);
}

#[test]
fn tuple_test() {
    let mut x = TupleTest::uninit();
    x.0.write("Here is some text");
}

#[test]
fn zst_test() {
    assert_eq!(::core::mem::size_of::<ZSTTest>(), 0);
    let x = ZSTTest::uninit();
    let x = unsafe { x.assume_init() };
    dbg!(x);
}

#[derive(Debug, DeepMaybeUninit)]
#[repr(transparent)]
struct Weird<'a, T> {
    x: &'a mut T,
}

#[test]
fn weird() {
    let mut x = Weird::uninit();
    let mut y = 0;
    x.x.write(&mut y);
    let x = unsafe { x.assume_init() };
    *x.x = 3;
    assert_eq!(y, 3);
}
