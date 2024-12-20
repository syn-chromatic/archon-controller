#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]
#![feature(async_closure)]
#![feature(arbitrary_self_types)]
#![feature(trait_alias)]
#![feature(impl_trait_in_assoc_type)]

pub mod consts;
pub mod devices;
pub mod diagnostics;
pub mod discovery;
pub mod endpoint;
pub mod input;
pub mod ring;
pub mod status;
pub mod rwlock;
pub mod utils;
pub mod socket;
