use std::any::type_name;

use crate::{Error, Result};

#[derive(Debug)]
pub enum ValueType {
    Null,
    Integer,
    Float,
    Text,
}

#[derive(Debug)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
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
        Value::Text(self.to_string())
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Value::Text(self.clone())
    }
}

pub trait FromValue: Sized {
    fn from_value(v: Value) -> Result<Self>;
}

macro_rules! impl_from_value_integer {
    ($ty:ident) => {
        impl FromValue for $ty {
            fn from_value(v: Value) -> Result<Self> {
                match v {
                    Value::Integer(i) => Ok(i as $ty),
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
            Value::Text(s) => Ok(s),
            _ => Err(Error::FromValue(format!(
                "Value type mismatch, cannot convert `{:?}` to {}",
                v,
                type_name::<Self>()
            ))),
        }
    }
}
