use serde::{ser, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
}

impl ser::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

pub(super) type Result<T> = std::result::Result<T, Error>;

/// Struct responsible for serializing to custom data format.
#[derive(Debug, Default)]
pub struct Serializer(String);

impl Serializer {
    /// Creates a new instance of `Serializer`.
    /// Initializes it with empty string buffer with capacity of `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    /// Finishes serialization and returns serialized data.
    pub fn finish(self) -> String {
        self.0
    }

    fn ensure_new_line(&mut self) {
        if !self.0.is_empty() && !self.0.ends_with('\n') {
            if self.0.ends_with(' ') {
                self.0.pop();
            }
            self.0.push('\n');
        }
    }

    fn ensure_white_space(&mut self) {
        if !self.0.ends_with(' ') && !self.0.ends_with('\n') && !self.0.is_empty() {
            self.0.push(' ');
        }
    }

    fn add_to_buffer<T: ToString>(&mut self, value: &T) {
        self.0.push_str(&value.to_string());
    }

    fn whitespace_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.ensure_white_space();
        value.serialize(&mut *self)
    }

    fn new_line_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.ensure_new_line();
        value.serialize(&mut *self)
    }
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> Result<()> {
        self.add_to_buffer(&value);
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<()> {
        self.serialize_i64(value.into())
    }

    fn serialize_i16(self, value: i16) -> Result<()> {
        self.serialize_i64(value.into())
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        self.serialize_i64(value.into())
    }

    fn serialize_i64(self, value: i64) -> Result<()> {
        self.add_to_buffer(&value);
        Ok(())
    }

    fn serialize_i128(self, value: i128) -> Result<()> {
        self.add_to_buffer(&value);
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        self.serialize_u64(value.into())
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        self.serialize_u64(value.into())
    }

    fn serialize_u32(self, value: u32) -> Result<()> {
        self.serialize_u64(value.into())
    }

    fn serialize_u64(self, value: u64) -> Result<()> {
        self.add_to_buffer(&value);
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> Result<()> {
        self.add_to_buffer(&value);
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        self.serialize_f64(value.into())
    }

    fn serialize_f64(self, value: f64) -> Result<()> {
        self.add_to_buffer(&value);
        Ok(())
    }

    fn serialize_char(self, value: char) -> Result<()> {
        self.0.push(value);
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        self.add_to_buffer(&value);
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        for byte in value {
            self.serialize_str(&format!("{byte:02X}"))?;
        }
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.0.push('-');
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        self.serialize_str(variant)?;
        self.ensure_white_space();
        value.serialize(self)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self> {
        Ok(self)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self> {
        Ok(self)
    }

    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self> {
        self.serialize_str(variant)?;
        Ok(self)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self> {
        Ok(self)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self> {
        self.serialize_str(variant)?;
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.new_line_field(value)
    }

    fn end(self) -> Result<()> {
        self.ensure_new_line();
        self.0.push('\n');
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.whitespace_field(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.whitespace_field(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.whitespace_field(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.new_line_field(key)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.whitespace_field(value)
    }

    fn end(self) -> Result<()> {
        self.ensure_new_line();
        self.0.push('\n');
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &'static str, value: &T) -> Result<()> {
        self.whitespace_field(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &'static str, value: &T) -> Result<()> {
        self.whitespace_field(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;
    use serde::Serializer as SerializerTrait;
    use std::collections::BTreeMap;

    macro_rules! test {
        ($ty:ty, $input:expr, $value:literal) => {
            let mut serializer = Serializer::default();
            <$ty>::serialize(&$input, &mut serializer).unwrap();
            assert_eq!(serializer.0, $value);
        };
    }

    #[test]
    fn serialize_bool() {
        test!(bool, true, "true");
        test!(bool, false, "false");
    }

    #[test]
    fn serialize_i8() {
        test!(i8, 1, "1");
    }

    #[test]
    fn serialize_i16() {
        test!(i16, 1, "1");
    }

    #[test]
    fn serialize_i32() {
        test!(i32, 1, "1");
    }

    #[test]
    fn serialize_i64() {
        test!(i64, 1, "1");
    }

    #[test]
    fn serialize_i128() {
        test!(i128, 1, "1");
    }

    #[test]
    fn serialize_u8() {
        test!(u8, 1, "1");
    }

    #[test]
    fn serialize_u16() {
        test!(u16, 1, "1");
    }

    #[test]
    fn serialize_u32() {
        test!(u32, 1, "1");
    }

    #[test]
    fn serialize_u64() {
        test!(u64, 1, "1");
    }

    #[test]
    fn serialize_u128() {
        test!(u128, 1, "1");
    }

    #[test]
    fn serialize_f32() {
        test!(f32, 1.5, "1.5");
    }

    #[test]
    fn serialize_f64() {
        test!(f64, 1.5, "1.5");
    }

    #[test]
    fn serialize_char() {
        test!(char, 'a', "a");
    }

    #[test]
    fn serialize_str() {
        test!(&str, "abc", "abc");
    }

    #[test]
    fn serialize_bytes() {
        let mut serializer = Serializer::default();
        assert!(serializer.serialize_bytes(&[0x01, 0x02, 0xab]).is_ok());
        assert_eq!(serializer.0, "0102AB");
    }

    #[test]
    fn serialize_none() {
        test!(Option<i32>, None, "-");
    }

    #[test]
    fn serialize_some() {
        test!(Option<i32>, Some(1), "1");
    }

    #[test]
    fn serialize_unit() {
        test!((), (), "-");
    }

    #[test]
    fn serialize_unit_struct() {
        test!(UnitStruct, UnitStruct, "-");
    }

    #[test]
    fn serialize_newtype_struct() {
        test!(NewType, NewType(1), "1");
    }

    #[test]
    fn serialize_newtype_variant() {
        test!(Enum, Enum::NewType(1), "NewType 1");
    }

    #[test]
    fn serialize_seq() {
        test!(Vec<i32>, vec![1, 2, 3], "1\n2\n3\n\n");
    }

    #[test]
    fn serialize_tuple() {
        test!((u8, u8, u8), (1, 2, 3), "1 2 3");
    }

    #[test]
    fn serialize_tuple_struct() {
        test!(TupleStruct, TupleStruct(1, 2, 3), "1 2 3");
    }

    #[test]
    fn serialize_unit_variant() {
        test!(Enum, Enum::Unit, "Unit");
    }

    #[test]
    fn serialize_tuple_variant() {
        test!(Enum, Enum::Tuple(1, 2, 3), "Tuple 1 2 3");
    }

    #[test]
    fn serialize_map() {
        test!(BTreeMap<i32, i32>, BTreeMap::from([(1, 2), (3, 4)]), "1 2\n3 4\n\n");
    }

    #[test]
    fn serialize_struct() {
        test!(Struct, Struct { a: 1, b: 2, c: 3 }, "1 2 3");
    }

    #[test]
    fn serialize_struct_variant() {
        test!(Enum, Enum::Struct { a: 1, b: 2, c: 3 }, "Struct 1 2 3");
    }

    #[test]
    fn serialize_advanced_struct() {
        test!(
            Advanced,
            new_advanced_struct(),
            "1 2\n3 4 5\n6 7 8\n\n9 Unit\n10 Tuple 11 12 13\n\n14 15 16 17 18"
        );
    }
}
