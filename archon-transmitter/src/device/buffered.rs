#![allow(dead_code)]
use super::DevicesBuilder;

use archon_core::devices::layout::DeviceLayout;
use archon_core::input::InputType;
use archon_core::ring::AtomicRingBuffer;
use archon_core::rwlock::RwLock;
use archon_core::rwlock::RwLockReadGuard;

use embsys::crates::embassy_futures;
use embsys::crates::embassy_sync;
use embsys::exts::std;

use std::vec::Vec;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex as CsMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::waitqueue::WakerRegistration;

const RWLOCK_SIZE: usize = 2;
const INPUT_BUFFER: usize = 512;

pub static BUFF_DEVICE: RwLockBufferedDevice = RwLock::new(BufferedDeviceLayout::new());

// BufferedDeviceLayout
type RwLockBufferedDevice = RwLock<CsMutex, BufferedDeviceLayout, RWLOCK_SIZE>;
type RwReadBufferedDevice<'a> = RwLockReadGuard<'a, CsMutex, BufferedDeviceLayout, RWLOCK_SIZE>;

// DeviceLayout
type MutexDeviceLayout = Mutex<CsMutex, DeviceLayout>;

// AtomicRingBuffer
type RwLockRingBuffer =
    RwLock<CsMutex, AtomicRingBuffer<Vec<InputType>, INPUT_BUFFER>, RWLOCK_SIZE>;

pub struct BufferedDeviceLayout {
    layout: MutexDeviceLayout,
    ring: RwLockRingBuffer,
}

impl BufferedDeviceLayout {
    async fn get_layout_inputs(&self) -> Vec<InputType> {
        let mut layout: _ = self.layout.lock().await;
        let inputs: Vec<InputType> = layout.get_inputs().await;
        inputs
    }

    async fn read_lock<'a>() -> RwReadBufferedDevice<'a> {
        BUFF_DEVICE.read().await
    }
}

impl BufferedDeviceLayout {
    pub const fn new() -> Self {
        let layout: _ = Mutex::new(DeviceLayout::new());
        let ring: _ = RwLock::new(AtomicRingBuffer::new());

        Self { layout, ring }
    }

    pub async fn build_layout() {
        let s: _ = Self::read_lock().await;

        let mut layout: _ = s.layout.lock().await;
        DevicesBuilder::build(&mut layout).await;
    }

    pub async fn collect() {
        let s: _ = Self::read_lock().await;

        loop {
            embassy_futures::yield_now().await;
            let inputs: Vec<InputType> = s.get_layout_inputs().await;
            if !inputs.is_empty() {
                s.ring.write().await.add(inputs);
            }
        }
    }

    pub async fn take_inputs() -> Vec<InputType> {
        let s: _ = Self::read_lock().await;

        if let Some(inputs) = s.ring.read().await.take_clone() {
            return inputs;
        }

        Vec::new()
    }
}
