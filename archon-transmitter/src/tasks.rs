use crate::consts::WIFI_PASS;
use crate::consts::WIFI_SSID;
use crate::transmitter::ArchonTransmitter;

use embsys::crates::embassy_executor;
use embsys::crates::embassy_time;
use embsys::drivers;
use embsys::helpers;

use helpers::task_handler::TaskState;
use helpers::wpa_psk::WpaPsk;

use drivers::hardware::WIFIController;
use embassy_time::Duration;

#[embassy_executor::task]
pub async fn archon_send(_s: TaskState) {
    let _ = ArchonTransmitter::read_lock().send().await;
}

#[embassy_executor::task]
pub async fn archon_collect(_s: TaskState) {
    let _ = ArchonTransmitter::read_lock().collect().await;
}

#[embassy_executor::task]
pub async fn wifi_connect(_s: TaskState) {
    let wifi_controller: &mut WIFIController = WIFIController::borrow_mut();

    let psk: [u8; 32] = WpaPsk::new().derive_psk(WIFI_SSID, WIFI_PASS);
    let timeout: Duration = Duration::from_secs(30);
    let _ = wifi_controller
        .join_wpa2_psk(WIFI_SSID, &psk, timeout)
        .await;
}
