use std::{any::type_name, ffi::CString};

use crate::{Error, Result};

/// (year, month, day, hour, minute, second, microsecond)
pub type DateTimeTuple = (u16, u8, u8, u8, u8, u8, u32);

#[derive(Debug, PartialEq, Eq)]
pub enum ValueType {
    Null,
    Integer,
    Float,
    Text,
    Blob,
    DateTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Text(CString),
    Blob(Vec<u8>),
    /// (year, month, day, hour, minute, second, microsecond)
    DateTime(u16, u8, u8, u8, u8, u8, u32),
}

pub trait ToValue {
    fn to_value(&self) -> Value;
}

macro_rules! impl_to_value_integer {
    ($ty:ident) => {
        impl ToValue for $ty {
            fn to_value(&self) -> Value {
                Value::Integer(*self as i64)
            }
        }
    };
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Value::Integer(if *self { 1 } else { 0 })
    }
}

impl_to_value_integer!(i8);
impl_to_value_integer!(u8);
impl_to_value_integer!(i16);
impl_to_value_integer!(u16);
impl_to_value_integer!(i32);
impl_to_value_integer!(u32);
impl_to_value_integer!(i64);
impl_to_value_integer!(u64);
impl_to_value_integer!(isize);
impl_to_value_integer!(usize);

impl ToValue for f32 {
    fn to_value(&self) -> Value {
        Value::Float(*self as f64)
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Value::Float(*self as f64)
    }
}

impl ToValue for &str {
    fn to_value(&self) -> Value {
        Value::Text(CString::new(self.to_string()).unwrap_or(CString::default()))
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Value::Text(CString::new(self.clone()).unwrap_or(CString::default()))
    }
}

impl ToValue for &[u8] {
    fn to_value(&self) -> Value {
        Value::Blob(self.to_vec())
    }
}

impl ToValue for Vec<u8> {
    fn to_value(&self) -> Value {
        Value::Blob(self.clone())
    }
}

impl ToValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl ToValue for DateTimeTuple {
    fn to_value(&self) -> Value {
        Value::DateTime(self.0, self.1, self.2, self.3, self.4, self.5, self.6)
    }
}

pub trait FromValue: Sized {
    fn from_value(v: Value) -> Result<Self>;
}

impl FromValue for bool {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Integer(i) => Ok(i != 0),
            _ => Err(Error::FromValue(format!(
                "Value type mismatch, cannot convert `{:?}` to {}",
                v,
                type_name::<Self>()
            ))),
        }
    }
}

macro_rules! impl_from_value_integer {
    ($ty:ident) => {
        impl FromValue for $ty {
            fn from_value(v: Value) -> Result<Self> {
                match v {
                    Value::Integer(i) => Ok(i as $ty),
                    Value::Float(f) => Ok(f as $ty),
                    _ => Err(Error::FromValue(format!(
                        "Value type mismatch, cannot convert `{:?}` to {}",
                        v,
                        type_name::<Self>()
                    ))),
                }
            }
        }
    };
}

impl_from_value_integer!(i8);
impl_from_value_integer!(u8);
impl_from_value_integer!(i16);
impl_from_value_integer!(u16);
impl_from_value_integer!(i32);
impl_from_value_integer!(u32);
impl_from_value_integer!(i64);
impl_from_value_integer!(u64);
impl_from_value_integer!(isize);
impl_from_value_integer!(usize);

impl FromValue for f32 {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Integer(i) => Ok(i as f32),
            Value::Float(f) => Ok(f as f32),
            _ => Err(Error::FromValue(format!(
                "Value type mismatch, cannot convert `{:?}` to {}",
                v,
                type_name::<Self>()
            ))),
        }
    }
}

impl FromValue for f64 {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Integer(i) => Ok(i as f64),
            Value::Float(f) => Ok(f),
            _ => Err(Error::FromValue(format!(
                "Value type mismatch, cannot convert `{:?}` to {}",
                v,
                type_name::<Self>()
            ))),
        }
    }
}

impl FromValue for String {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Text(s) => Ok(s
                .into_string()
                .map_err(|e| Error::FromValue(format!("CString to String error: {}", e)))?),
            _ => Err(Error::FromValue(format!(
                "Value type mismatch, cannot convert `{:?}` to {}",
                v,
                type_name::<Self>()
            ))),
        }
    }
}

impl FromValue for Vec<u8> {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::Text(s) => {
                let s = s
                    .into_string()
                    .map_err(|e| Error::FromValue(format!("CString to String error: {}", e)))?;
                Ok(s.into_bytes())
            }
            Value::Blob(v) => Ok(v),
            _ => Err(Error::FromValue(format!(
                "Value type mismatch, cannot convert `{:?}` to {}",
                v,
                type_name::<Self>()
            ))),
        }
    }
}

impl FromValue for DateTimeTuple {
    fn from_value(v: Value) -> Result<Self> {
        match v {
            Value::DateTime(year, month, day, hour, minute, second, nanosecond) => {
                Ok((year, month, day, hour, minute, second, nanosecond))
            }
            _ => Err(Error::FromValue(format!(
                "Value type mismatch, cannot convert `{:?}` to {}",
                v,
                type_name::<Self>()
            ))),
        }
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(v: Value) -> Result<Self> {
        if let Value::Null = v {
            Ok(None)
        } else {
            Ok(Some(T::from_value(v)?))
        }
    }
}
