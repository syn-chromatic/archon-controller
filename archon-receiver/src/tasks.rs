use crate::consts::WIFI_PASS;
use crate::consts::WIFI_SSID;
use crate::receiver::ArchonReceiver;

use embsys::crates::embassy_executor;
use embsys::crates::embassy_net;
use embsys::crates::embassy_time;
use embsys::crates::heapless;
use embsys::drivers;
use embsys::helpers;

use heapless::Vec;

use helpers::task_handler::TaskState;
use helpers::wpa_psk::WpaPsk;

use drivers::hardware::WIFIController;
use embassy_net::ConfigV4;
use embassy_net::Ipv4Address;
use embassy_net::Ipv4Cidr;
use embassy_net::StaticConfigV4;
use embassy_time::Duration;

#[embassy_executor::task]
pub async fn archon_listen(_s: TaskState) {
    let _ = ArchonReceiver::read_lock().listen().await;
}

#[embassy_executor::task]
pub async fn wifi_connect(_s: TaskState) {
    let wifi_controller: &mut WIFIController = WIFIController::as_mut();

    let psk: [u8; 32] = WpaPsk::new().derive_psk(WIFI_SSID, WIFI_PASS);
    let timeout: Duration = Duration::from_secs(60);
    let _ = wifi_controller
        .join_wpa2_psk(WIFI_SSID, &psk, timeout)
        .await;
}

#[embassy_executor::task]
pub async fn wifi_connect_static(_s: TaskState) {
    let wifi_controller: &mut WIFIController = WIFIController::as_mut();

    let psk: [u8; 32] = WpaPsk::new().derive_psk(WIFI_SSID, WIFI_PASS);
    let address: Ipv4Cidr = Ipv4Cidr::new(Ipv4Address::new(192, 168, 0, 140), 24);
    let gateway: Ipv4Address = Ipv4Address::new(192, 168, 0, 1);

    let mut dns_servers: Vec<Ipv4Address, 3> = Vec::new();
    dns_servers.push(Ipv4Address::new(192, 168, 0, 1)).unwrap();

    let static_config = StaticConfigV4 {
        address,
        gateway: Some(gateway),
        dns_servers,
    };

    let config = ConfigV4::Static(static_config);
    let _ = wifi_controller
        .join_wpa2_psk_static(WIFI_SSID, &psk, config)
        .await;
}
