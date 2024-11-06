#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]
#![feature(async_closure)]
#![feature(arbitrary_self_types)]
#![feature(trait_alias)]
#![feature(impl_trait_in_assoc_type)]
#![allow(static_mut_refs)]

extern crate embsys;
use embsys::crates::emballoc::Allocator;

#[global_allocator]
pub static ALLOCATOR: Allocator<163_840> = Allocator::new();

mod consts;
mod devices;
mod display;
mod entry;
mod interface;
mod tasks;
mod tests;
mod transmitter;
