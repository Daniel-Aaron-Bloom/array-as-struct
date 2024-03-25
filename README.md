# Array As Struct

A crate to make ergonomic "field" accesses on array types.

##  Motivation 

Sometimes you're working with APIs which take in slices or arrays, but your code would better served by named fields. An example of this might be an API which takes `[u8; 3]` for colors, but you'd prefer to have
```rust
struct Color{ r: u8, g: u8, b: u8 }
```

One solution is to use `repr(C)` and `transmute` or a `union`, but that obviously precludes `forbid(unsafe_code)`.

A safer option is to instead use an enum to represent each field, which can be done with crates like [`enum-map`](https://codeberg.org/xfix/enum-map). This is much better, but involves using macros to construct the type, and sometimes the indexing operations and enum-variant resolution obstruct the readability of code.

```rust
use enum_map::{EnumMap, Enum};

#[derive(Debug, Enum)]
enum ColorType {
    R,
    G,
    B,
}

type Color = EnumMap<ColorType, u8>;

let mut color: Color = enum_map! {
    Example::R => 247,
    Example::G => 0,
    Example::B => 0,
};
color[Example::G] = 76;
```

This crate provides an alternative approach, relying on helper structs to bridge the gap between the array type and the field-based struct type.

```rust
use array_as_struct::{array_as_struct, ArrayStruct};

#[array_as_struct]
// Despite this declaration, `Color` is actual a tuple struct
// which looks like `struct Color([u8; 3]);`
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

// This is the actual named-field struct
type ColorValue = <Color as ArrayStruct>::Value;

let mut color = Color::from_val(ColorValue { r: 247, g: 0, b: 0 });
*f.muts().g = 76;
```

## FAQ

* Why replace the original declaration with a array-based tuple struct? Why not just add the array-struct as a `derive`d type?

The main motivation of this crate is to help people deal with `[T]` APIs. By hiding the original field-based struct, this crate makes the array-based type the default, encouraging users to treat the field-based struct purely as a short-lived ergonomic tool for referencing "fields" by name.
