use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use std::collections::VecDeque;
use std::io::BufRead;
use thiserror::Error;

/// Enum representing deserializing errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("expected a unit value '-'")]
    ExpectedUnit,
    #[error("deserialization any type is not supported")]
    AnyNotSupported,
    #[error("invalid hex length (must be multiple of 2)")]
    InvalidHexLength,
    #[error("encountered an unexpected empty line")]
    UnexpectedEmptyLine,
    #[error("encountered an unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("{0}")]
    Custom(String),
    #[error("IO error: {0}")]
    IOErr(#[from] std::io::Error),
    #[error("parse bool error: {0}")]
    ParseBoolErr(#[from] std::str::ParseBoolError),
    #[error("parse int error: {0}")]
    ParseIntErr(#[from] std::num::ParseIntError),
    #[error("parse float error: {0}")]
    ParseFloatErr(#[from] std::num::ParseFloatError),
    #[error("parse char error: {0}")]
    ParseCharErr(#[from] std::char::ParseCharError),
}

impl serde::de::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

pub(super) type Result<T> = std::result::Result<T, Error>;

/// Struct responsible for deserializing data from a file.
#[derive(Debug)]
pub struct Deserializer<'a, R: BufRead> {
    source: &'a mut R,
    buffer: VecDeque<String>,
}

impl<'a, R: BufRead> Deserializer<'a, R> {
    /// Creates a new deserializer.
    pub fn new(source: &'a mut R) -> Self {
        Self {
            source,
            buffer: VecDeque::new(),
        }
    }

    fn next(&mut self) -> Result<String> {
        if self.buffer.is_empty() {
            self.load_line()?;
        }

        self.buffer.pop_front().ok_or(Error::UnexpectedEmptyLine)
    }

    fn peek_next(&mut self) -> Result<&String> {
        if self.buffer.is_empty() {
            self.load_line()?;
        }

        self.buffer.front().ok_or(Error::UnexpectedEmptyLine)
    }

    fn load_line(&mut self) -> Result<()> {
        let mut line = String::new();

        if self.source.read_line(&mut line)? == 0 {
            return Err(Error::UnexpectedEndOfInput);
        }

        let trimmed = line.trim();

        for line in trimmed.split(' ') {
            self.buffer.push_back(line.to_string());
        }

        Ok(())
    }

    fn parse_next_element<T: DeserializeSeed<'a>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.peek_next()?.is_empty() {
            self.next()?;
            Ok(None)
        } else {
            seed.deserialize(&mut *self).map(Some)
        }
    }
}

impl<'a, R: BufRead> serde::de::Deserializer<'a> for &mut Deserializer<'a, R> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'a>>(self, _: V) -> Result<V::Value> {
        Err(Error::AnyNotSupported)
    }

    fn deserialize_bool<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_bool(self.next()?.parse()?)
    }

    fn deserialize_i8<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.next()?.parse()?)
    }

    fn deserialize_i16<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(self.next()?.parse()?)
    }

    fn deserialize_i32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.next()?.parse()?)
    }

    fn deserialize_i64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.next()?.parse()?)
    }

    fn deserialize_i128<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i128(self.next()?.parse()?)
    }

    fn deserialize_u8<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.next()?.parse()?)
    }

    fn deserialize_u16<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(self.next()?.parse()?)
    }

    fn deserialize_u32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.next()?.parse()?)
    }

    fn deserialize_u64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(self.next()?.parse()?)
    }

    fn deserialize_u128<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u128(self.next()?.parse()?)
    }

    fn deserialize_f32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.next()?.parse()?)
    }

    fn deserialize_f64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.next()?.parse()?)
    }

    fn deserialize_char<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_char(self.next()?.parse()?)
    }

    fn deserialize_str<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_string(self.next()?)
    }

    fn deserialize_string<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_string(self.next()?)
    }

    fn deserialize_bytes<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        let value = self.next()?;

        if value.len() & 1 == 0 {
            return Err(Error::InvalidHexLength);
        }

        let capacity = value.len() / 2;
        let mut buffer = Vec::with_capacity(capacity);

        for byte in 0..capacity {
            let index = byte * 2;
            buffer.push(u8::from_str_radix(&value[index..(index + 2)], 16)?);
        }

        visitor.visit_byte_buf(buffer)
    }

    fn deserialize_option<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        if self.peek_next()? == "-" {
            self.next()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        if self.next()? == "-" {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedUnit)
        }
    }

    fn deserialize_unit_struct<V: Visitor<'a>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'a>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V: Visitor<'a>>(self, _: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(SimpleSeqAccess(self))
    }

    fn deserialize_tuple_struct<V: Visitor<'a>>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_seq(SimpleSeqAccess(self))
    }

    fn deserialize_map<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V: Visitor<'a>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_seq(SimpleSeqAccess(self))
    }

    fn deserialize_enum<V: Visitor<'a>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(visitor)
    }
}

/// Struct responsible for deserializing a sequence of data to structs and tuples.
struct SimpleSeqAccess<'a, 'b, R: BufRead>(&'b mut Deserializer<'a, R>);

impl<'a, 'b, R: BufRead> SeqAccess<'a> for SimpleSeqAccess<'a, 'b, R> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'a>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        seed.deserialize(&mut *self.0).map(Some)
    }
}

impl<'a, R: BufRead> SeqAccess<'a> for &mut Deserializer<'a, R> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'a>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        self.parse_next_element(seed)
    }
}

impl<'a, R: BufRead> MapAccess<'a> for &mut Deserializer<'a, R> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'a>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        self.parse_next_element(seed)
    }

    fn next_value_seed<V: DeserializeSeed<'a>>(&mut self, seed: V) -> Result<V::Value> {
        seed.deserialize(&mut **self)
    }
}

impl<'a, R: BufRead> VariantAccess<'a> for &mut Deserializer<'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'a>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'a>>(self, _: usize, seed: V) -> Result<V::Value> {
        seed.visit_seq(SimpleSeqAccess(self))
    }

    fn struct_variant<V: Visitor<'a>>(
        self,
        _: &'static [&'static str],
        seed: V,
    ) -> Result<V::Value> {
        seed.visit_seq(SimpleSeqAccess(self))
    }
}

impl<'a, R: BufRead> EnumAccess<'a> for &mut Deserializer<'a, R> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: DeserializeSeed<'a>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        Ok((seed.deserialize(&mut *self)?, self))
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::Error::UnexpectedEndOfInput as EOF;
    use super::Error::*;
    use super::*;
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use std::io::Cursor;

    macro_rules! test {
        ($ty:ty, $input:literal $(, $value:expr)+) => {
            let mut input = Cursor::new($input);
            let mut deserializer = Deserializer::new(&mut input);
            $(assert_eq!($value, <$ty>::deserialize(&mut deserializer).unwrap());)+
        };
        (err, $ty:ty, $input:literal $(, $value:pat)+) => {
            let mut input = Cursor::new($input);
            let mut deserializer = Deserializer::new(&mut input);
            $(assert!(matches!(<$ty>::deserialize(&mut deserializer), Err($value)));)+
        };
    }

    #[test]
    fn deserialize_bool() {
        test!(bool, "true false", true, false);
    }

    #[test]
    fn deserialize_bool_error() {
        test!(err, bool, "a", _err, EOF);
    }

    #[test]
    fn deserialize_i8() {
        test!(i8, "1 -1", 1, -1);
    }

    #[test]
    fn deserialize_i8_error() {
        test!(err, i8, "a 128", ParseIntErr(_), ParseIntErr(_), EOF);
    }

    #[test]
    fn deserialize_i16() {
        test!(i16, "1 -1", 1, -1);
    }

    #[test]
    fn deserialize_i16_error() {
        test!(err, i16, "a 32769", ParseIntErr(_), ParseIntErr(_), EOF);
    }

    #[test]
    fn deserialize_i32() {
        test!(i32, "1 -1", 1, -1);
    }

    #[test]
    fn deserialize_i32_error() {
        test!(
            err,
            i32,
            "a 2147483649",
            ParseIntErr(_),
            ParseIntErr(_),
            EOF
        );
    }

    #[test]
    fn deserialize_i64() {
        test!(i64, "1 -1", 1, -1);
    }

    #[test]
    fn deserialize_i64_error() {
        test!(
            err,
            i64,
            "a 9223372036854775809",
            ParseIntErr(_),
            ParseIntErr(_),
            EOF
        );
    }

    #[test]
    fn deserialize_i128() {
        test!(i128, "1 -1", 1, -1);
    }

    #[test]
    fn deserialize_i128_error() {
        test!(
            err,
            i128,
            "a 170141183460469231731687303715884105729",
            ParseIntErr(_),
            ParseIntErr(_),
            EOF
        );
    }

    #[test]
    fn deserialize_u8() {
        test!(u8, "1 255", 1, 255);
    }

    #[test]
    fn deserialize_u8_error() {
        test!(err, u8, "a 256", ParseIntErr(_), ParseIntErr(_), EOF);
    }

    #[test]
    fn deserialize_u16() {
        test!(u16, "1 65535", 1, 65535);
    }

    #[test]
    fn deserialize_u16_error() {
        test!(err, u16, "a 65536", ParseIntErr(_), ParseIntErr(_), EOF);
    }

    #[test]
    fn deserialize_u32() {
        test!(u32, "1 4294967295", 1, 4_294_967_295);
    }

    #[test]
    fn deserialize_u32_error() {
        test!(
            err,
            u32,
            "a 4294967296",
            ParseIntErr(_),
            ParseIntErr(_),
            EOF
        );
    }

    #[test]
    fn deserialize_u64() {
        test!(u64, "1 18446744073709551615", 1, 18_446_744_073_709_551_615);
    }

    #[test]
    fn deserialize_u64_error() {
        test!(
            err,
            u64,
            "a 18446744073709551616",
            ParseIntErr(_),
            ParseIntErr(_),
            EOF
        );
    }

    #[test]
    fn deserialize_u128() {
        test!(
            u128,
            "1 340282366920938463463374607431768211455",
            1,
            340_282_366_920_938_463_463_374_607_431_768_211_455
        );
    }

    #[test]
    fn deserialize_u128_error() {
        test!(
            err,
            u128,
            "a 340282366920938463463374607431768211456",
            ParseIntErr(_),
            ParseIntErr(_),
            EOF
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn deserialize_f32() {
        test!(f32, "1.5 -1.5", 1.5, -1.5);
    }

    #[test]
    fn deserialize_f32_error() {
        test!(err, f32, "a", ParseFloatErr(_), EOF);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn deserialize_f64() {
        test!(f64, "1.5 -1.5", 1.5, -1.5);
    }

    #[test]
    fn deserialize_f64_error() {
        test!(err, f64, "a", ParseFloatErr(_), EOF);
    }

    #[test]
    fn deserialize_char() {
        test!(char, "a b", 'a', 'b');
    }

    #[test]
    fn deserialize_char_error() {
        test!(err, char, "ab", ParseCharErr(_), EOF);
    }

    #[test]
    fn deserialize_str() {
        test!(String, "ab cd", "ab", "cd");
    }

    #[test]
    fn deserialize_str_error() {
        test!(err, String, "", EOF);
    }

    #[test]
    fn deserialize_option() {
        test!(Option<i32>, "- 1 -", None, Some(1), None);
    }

    #[test]
    fn deserialize_option_error() {
        test!(err, Option<i32>, "a", ParseIntErr(_), EOF);
    }

    #[test]
    fn deserialize_unit() {
        test!((), "- -", (), ());
    }

    #[test]
    fn deserialize_unit_error() {
        test!(err, (), "a", ExpectedUnit, EOF);
    }

    #[test]
    fn deserialize_newtype_struct() {
        test!(NewType, "1 2", NewType(1), NewType(2));
    }

    #[test]
    fn deserialize_seq() {
        test!(
            Vec<i32>,
            "1\n2\n3\n\n4\n5\n6\n\n",
            vec![1, 2, 3],
            vec![4, 5, 6]
        );
    }

    #[test]
    fn deserialize_tuple_struct() {
        test!(
            TupleStruct,
            "1 2 3 4 5 6",
            TupleStruct(1, 2, 3),
            TupleStruct(4, 5, 6)
        );
    }

    #[test]
    fn deserialize_map() {
        test!(
            BTreeMap<i32, i32>,
            "1 2\n3 4\n5 6\n\n7 8\n9 10\n11 12\n\n",
            BTreeMap::from([(1, 2), (3, 4), (5, 6)]),
            BTreeMap::from([(7, 8), (9, 10), (11, 12)])
        );
    }

    #[test]
    fn deserialize_struct() {
        test!(
            Struct,
            "1 2 3 4 5 6",
            Struct { a: 1, b: 2, c: 3 },
            Struct { a: 4, b: 5, c: 6 }
        );
    }

    #[test]
    fn deserialize_enum() {
        test!(
            Enum,
            "Unit Tuple 1 2 3 Struct 4 5 6",
            Enum::Unit,
            Enum::Tuple(1, 2, 3),
            Enum::Struct { a: 4, b: 5, c: 6 }
        );
    }

    #[test]
    fn deserialize_advanced_struct() {
        test!(
            Advanced,
            "1 2\n3 4 5\n6 7 8\n\n9 Unit\n10 Tuple 11 12 13\n\n14 15 16 17 18",
            new_advanced_struct()
        );
    }
}
