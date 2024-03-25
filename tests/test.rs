use array_as_struct::{array_as_struct, ArrayStruct};

#[array_as_struct]
#[derive(Clone)]
pub struct Foo {
    bar: u32,
    baz: u32,
}

#[test]
fn main() {
    // Workaround rust-lang/rust#86935
    type Value = <Foo as ArrayStruct>::Value;

    let mut f = Foo::from_val(Value { bar: 10, baz: 15 });

    assert_eq!(<Foo as ArrayStruct>::Index::bar(), 0);
    assert_eq!(<Foo as ArrayStruct>::Index::baz(), 1);
    assert_eq!(f.0, [10, 15]);
    f[<Foo as ArrayStruct>::Index::bar()] = 2;
    assert_eq!(*f.refs().bar, 2);
    *f.muts().baz = 12;
    assert_eq!(*f.refs().baz, 12);
}
