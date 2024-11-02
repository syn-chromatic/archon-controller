#![allow(dead_code)]

use crate::devices::DevicesBuilder;
use archon_core::devices::layout::DeviceLayout;

pub async fn test_device_layout() {
    let mut layout: DeviceLayout = DeviceLayout::new();
    DevicesBuilder::build(&mut layout).await;

    loop {
        for input in layout.get_inputs().await {
            input.defmt();
        }
    }
}
