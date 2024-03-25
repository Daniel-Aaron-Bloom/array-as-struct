//! Procedural macro to make array-types with simulated fields
#![forbid(unsafe_code)]
#![no_std]

/// A trait to name all the associated types and simplify the conversion to and
/// from the helper types.
pub trait ArrayStruct {
    /// Helper type which is identical to the original field-struct declaration
    type Value;
    /// The underlying array type
    type Array;
    /// Helper type which is similar to the original field-struct declaration,
    /// but with `&'a T` instead of `T` for the field type
    type Refs<'a>
    where
        Self: 'a;
    /// Helper type which is similar to the original field-struct declaration,
    /// but with `&'a mut T` instead of `T` for the field type
    type Muts<'a>
    where
        Self: 'a;

    /// Helper type which contains helper functions to get the index of each field
    /// by name.
    ///
    /// ```
    /// # use array_as_struct::{array_as_struct_doctest as array_as_struct, ArrayStruct};
    /// # mod _hider{
    /// use array_as_struct::{array_as_struct, ArrayStruct};
    /// # }
    ///
    /// #[array_as_struct]
    /// pub struct Foo {
    ///     bar: u32,
    ///     baz: u32,
    /// }
    ///
    /// assert_eq!(<Foo as ArrayStruct>::Index::bar(), 0);
    /// assert_eq!(<Foo as ArrayStruct>::Index::baz(), 1);
    /// ```
    type Index;

    /// Construct the tuple-struct type from the named-field type
    ///
    /// ```
    /// # use array_as_struct::{array_as_struct_doctest as array_as_struct, ArrayStruct};
    /// # mod _hider{
    /// use array_as_struct::{array_as_struct, ArrayStruct};
    /// # }
    ///
    /// #[array_as_struct]
    /// #[derive(Debug, PartialEq, Eq)]
    /// pub struct Foo {
    ///     bar: u32,
    ///     baz: u32,
    /// }
    ///
    /// // Workaround rust-lang/rust#86935
    /// type Value = <Foo as ArrayStruct>::Value;
    ///
    /// let f = Foo::from_val(Value { bar: 10, baz: 15 });
    ///
    /// assert_eq!(Foo([10, 15]), f);
    /// ```
    fn from_val(value: Self::Value) -> Self;

    /// Construct the named-field type from the tuple-struct type
    ///
    /// ```
    /// # use array_as_struct::{array_as_struct_doctest as array_as_struct, ArrayStruct};
    /// # mod _hider{
    /// use array_as_struct::{array_as_struct, ArrayStruct};
    /// # }
    ///
    /// #[array_as_struct]
    /// #[derive(Debug, PartialEq, Eq)]
    /// pub struct Foo {
    ///     bar: u32,
    ///     baz: u32,
    /// }
    ///
    /// // Workaround rust-lang/rust#86935
    /// type Value = <Foo as ArrayStruct>::Value;
    ///
    /// let f = Foo([10, 15]);
    ///
    /// assert_eq!(f.val(), Value { bar: 10, baz: 15 });
    fn val(self) -> Self::Value;

    /// Construct the tuple-struct type from the underlying array type
    ///
    /// This is basically the same as just calling the tuple-constructor
    ///
    /// ```
    /// # use array_as_struct::{array_as_struct_doctest as array_as_struct, ArrayStruct};
    /// # mod _hider{
    /// use array_as_struct::{array_as_struct, ArrayStruct};
    /// # }
    ///
    /// #[array_as_struct]
    /// #[derive(Debug, PartialEq, Eq)]
    /// pub struct Foo {
    ///     bar: u32,
    ///     baz: u32,
    /// }
    ///
    /// assert_eq!(Foo([1, 2]), <Foo as ArrayStruct>::from_array([1, 2]));
    /// ```
    fn from_array(array: Self::Array) -> Self;

    /// Construct the tuple-struct type from the underlying array type
    ///
    /// This is basically the same as just calling the `.0`
    ///
    /// ```
    /// # use array_as_struct::{array_as_struct_doctest as array_as_struct, ArrayStruct};
    /// # mod _hider{
    /// use array_as_struct::{array_as_struct, ArrayStruct};
    /// # }
    ///
    /// #[array_as_struct]
    /// #[derive(Copy, Clone)]
    /// pub struct Foo {
    ///     bar: u32,
    ///     baz: u32,
    /// }
    ///
    /// let f = Foo([1, 2]);
    ///
    /// assert_eq!(f.0, f.to_array());
    /// ```
    fn to_array(self) -> Self::Array;

    /// Construct the reference-named-field type from the tuple-struct type.
    ///
    /// This is the primary way of accessing the named fields
    ///
    fn refs(&'_ self) -> Self::Refs<'_>;

    /// Construct the mutable-reference-named-field type from the tuple-struct type
    fn muts(&'_ mut self) -> Self::Muts<'_>;
}

pub use array_as_struct_derive::array_as_struct;

#[doc(hidden)]
pub use array_as_struct_derive::array_as_struct_doctest;
