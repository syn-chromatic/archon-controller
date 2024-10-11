#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]
#![feature(type_alias_impl_trait)]
#![feature(async_closure)]
#![feature(arbitrary_self_types)]
#![feature(trait_alias)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embsys::crates::cortex_m_rt;
use embsys::crates::embassy_executor;

#[embassy_executor::main]
async fn entry(_spawner: Spawner) {}
