use crate::configuration::WIFI_PASS;
use crate::configuration::WIFI_SSID;
use crate::controller::ArchonReceiver;

use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_time;
use embsys::drivers;
use embsys::helpers;

use helpers::task_handler::TaskState;
use helpers::wpa_psk::WpaPsk;

use drivers::hardware::WIFIController;
use embassy_time::Duration;

// async fn dfu_cancel() {
//     let watchdog: &mut Watchdog = HWController::watchdog_mut();

//     while !SystemInterface::controller().activity().await {
//         embassy_futures::yield_now().await;
//         watchdog.feed();
//     }
// }

#[embassy_executor::task]
pub async fn archon(_s: TaskState) {
    defmt::info!("running dfu mode");

    let mut archon: ArchonReceiver = ArchonReceiver::new(None, 9688);
    let _ = archon.listen().await;
    // let _ = with_cancel(dfu.listen(), dfu_cancel()).await;
}

#[embassy_executor::task]
pub async fn connect_wifi(_s: TaskState) {
    let wifi_controller: &mut WIFIController = WIFIController::borrow_mut();

    let psk: [u8; 32] = WpaPsk::new().derive_psk(WIFI_SSID, WIFI_PASS);
    let timeout: Duration = Duration::from_secs(30);
    let _ = wifi_controller
        .join_wpa2_psk(WIFI_SSID, &psk, timeout)
        .await;
}
