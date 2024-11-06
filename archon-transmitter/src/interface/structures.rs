#![allow(unused_imports)]

use super::enums::BooleanEnum;
use super::enums::ValueEnum;

use embsys::crates::embassy_rp;
use embsys::crates::embassy_time;
use embsys::drivers::hardware::HWController;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::std;

use std::format;
use std::string::String;
use std::string::ToString;
use std::time::Duration;
use std::vec::Vec;

use embassy_rp::adc::Error as AdcError;
use embassy_time::with_timeout;
use embassy_time::TimeoutError;

use embedded_menu::items::menu_item::SelectValue;
use embedded_menu::items::MenuItem;

use archon_core::input::DPad;
use archon_core::input::InputType;

#[derive(Clone, PartialEq)]
pub struct SelectString {
    string: String,
}

impl SelectString {
    pub fn new(string: String) -> Self {
        Self { string }
    }
}

impl SelectValue for SelectString {
    fn marker(&self) -> &str {
        &self.string
    }
}

impl From<String> for SelectString {
    fn from(value: String) -> Self {
        SelectString::new(value)
    }
}

impl From<&str> for SelectString {
    fn from(value: &str) -> Self {
        SelectString::new(value.to_string())
    }
}

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
