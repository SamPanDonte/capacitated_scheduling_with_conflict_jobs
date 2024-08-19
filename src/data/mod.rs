mod de;
mod ser;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{stdin, BufRead};

/// Deserialize a value from buffered input.
pub fn deserialize<'de, I: BufRead, T: Deserialize<'de>>(input: &'de mut I) -> de::Result<T> {
    T::deserialize(&mut de::Deserializer::new(input))
}

/// Deserialize a value from stdin.
pub fn from_stdin<T: DeserializeOwned>() -> de::Result<T> {
    deserialize(&mut stdin().lock())
}

/// Serialize a value to string.
pub fn to_string<T: Serialize>(value: &T) -> ser::Result<String> {
    let mut serializer = ser::Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.finish())
}

/// Serialize a value to stdout.
pub fn to_stdout<T: Serialize>(value: &T) -> ser::Result<()> {
    println!("{}", to_string(value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::Cursor;

    #[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
    pub struct UnitStruct;

    #[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
    pub struct NewType(pub i32);

    #[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
    pub struct TupleStruct(pub i32, pub i32, pub i32);

    #[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
    pub enum Enum {
        #[default]
        Unit,
        NewType(i32),
        Tuple(i32, i32, i32),
        Struct {
            a: i32,
            b: i32,
            c: i32,
        },
    }

    #[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
    pub struct Struct {
        pub a: i32,
        pub b: i32,
        pub c: i32,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    pub struct Advanced {
        pub a: i32,
        pub b: String,
        pub c: Vec<Struct>,
        pub d: BTreeMap<u64, Enum>,
        pub e: (u64, u8, Struct),
    }

    impl Default for Advanced {
        fn default() -> Self {
            Self {
                a: 0,
                b: "default".to_string(),
                c: vec![],
                d: BTreeMap::new(),
                e: (0, 0, Struct::default()),
            }
        }
    }

    pub fn new_advanced_struct() -> Advanced {
        Advanced {
            a: 1,
            b: "2".to_string(),
            c: vec![Struct { a: 3, b: 4, c: 5 }, Struct { a: 6, b: 7, c: 8 }],
            d: BTreeMap::from([(9, Enum::Unit), (10, Enum::Tuple(11, 12, 13))]),
            e: (
                14,
                15,
                Struct {
                    a: 16,
                    b: 17,
                    c: 18,
                },
            ),
        }
    }

    macro_rules! test_impl {
        ($($(,)?$ty:ident)*) => {
            $(
                let value = $ty::default();
                assert_eq!(value, deserialize(&mut Cursor::new(to_string(&value).unwrap())).unwrap());
            )*
        };
    }

    #[test]
    fn serialize_and_deserialize() {
        test_impl!(UnitStruct, NewType, TupleStruct, Enum, Struct, Advanced);
    }
}
