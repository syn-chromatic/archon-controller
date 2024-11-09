#![allow(dead_code)]

use embsys::exts::std;

use std::format;
use std::string::String;
use std::string::ToString;

use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::SelectValue as SelectValueMacro;

#[derive(Copy, Clone, PartialEq)]
pub struct SubMenuSelect {
    index: Option<usize>,
}

impl SubMenuSelect {
    pub fn new(index: usize) -> Self {
        Self { index: Some(index) }
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }
}

impl Default for SubMenuSelect {
    fn default() -> Self {
        Self { index: None }
    }
}

impl SelectValue for SubMenuSelect {
    fn marker(&self) -> &str {
        ">"
    }
}

#[derive(Clone, PartialEq)]
pub struct U16Value {
    value: u16,
    value_str: String,
}

impl U16Value {
    pub fn new(value: u16) -> Self {
        Self {
            value,
            value_str: value.to_string(),
        }
    }
}

impl From<u16> for U16Value {
    fn from(value: u16) -> Self {
        U16Value::new(value)
    }
}

impl SelectValue for U16Value {
    fn marker(&self) -> &str {
        &self.value_str
    }
}

#[derive(Clone, PartialEq)]
pub struct F32Value {
    value: f32,
    value_str: String,
}

impl F32Value {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            value_str: format!("{:.2}", value),
        }
    }
}

impl From<f32> for F32Value {
    fn from(value: f32) -> Self {
        F32Value::new(value)
    }
}

impl SelectValue for F32Value {
    fn marker(&self) -> &str {
        &self.value_str
    }
}

#[derive(Clone, PartialEq)]
pub struct F32OpValue {
    value: Option<f32>,
    value_str: String,
}

impl F32OpValue {
    fn format(value: Option<f32>) -> String {
        if let Some(value) = value {
            return format!("{:.2}", value);
        }
        format!("Non")
    }
}

impl F32OpValue {
    pub fn new(value: Option<f32>) -> Self {
        Self {
            value,
            value_str: Self::format(value),
        }
    }
}

impl From<f32> for F32OpValue {
    fn from(value: f32) -> Self {
        F32OpValue::new(Some(value))
    }
}

impl SelectValue for F32OpValue {
    fn marker(&self) -> &str {
        &self.value_str
    }
}

#[derive(Copy, Clone, PartialEq, SelectValueMacro)]
pub enum BooleanEnum {
    ON,
    OFF,
}

impl BooleanEnum {
    pub fn new(state: bool) -> Self {
        match state {
            true => BooleanEnum::ON,
            false => BooleanEnum::OFF,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ValueEnum {
    F32(F32Value),
    F32Op(F32OpValue),
    U16(U16Value),
    Boolean(BooleanEnum),
    String(String),
    Str(&'static str),
}

impl ValueEnum {
    pub fn u16(value: u16) -> Self {
        ValueEnum::U16(U16Value::new(value))
    }

    pub fn f32(value: f32) -> Self {
        ValueEnum::F32(F32Value::new(value))
    }

    pub fn f32_op(value: Option<f32>) -> Self {
        ValueEnum::F32Op(F32OpValue::new(value))
    }

    pub fn boolean(value: bool) -> Self {
        ValueEnum::Boolean(BooleanEnum::new(value))
    }

    pub fn string(value: &str) -> ValueEnum {
        ValueEnum::String(value.to_string())
    }

    pub fn str(value: &'static str) -> ValueEnum {
        ValueEnum::Str(value)
    }

    pub fn empty() -> ValueEnum {
        ValueEnum::Str("")
    }

    pub fn arrow() -> ValueEnum {
        ValueEnum::Str(">")
    }
}

impl SelectValue for ValueEnum {
    fn marker(&self) -> &str {
        match self {
            ValueEnum::F32(f32) => f32.marker(),
            ValueEnum::F32Op(f32) => f32.marker(),
            ValueEnum::U16(u16) => u16.marker(),
            ValueEnum::Boolean(boolean) => boolean.marker(),
            ValueEnum::String(string) => string,
            ValueEnum::Str(str) => *str,
        }
    }
}

impl From<String> for ValueEnum {
    fn from(value: String) -> Self {
        ValueEnum::String(value)
    }
}

impl From<&'static str> for ValueEnum {
    fn from(value: &'static str) -> Self {
        ValueEnum::Str(value)
    }
}
