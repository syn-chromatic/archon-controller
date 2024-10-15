use crate::button_test::button_test;
use crate::tasks::wifi_connect;
use crate::transmitter::ArchonTransmitter;

use archon_core::endpoint::ArchonEndpoint;
use archon_core::input::InputType;

use embsys::crates::cortex_m_rt;
use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::crates::embassy_time;
use embsys::drivers::hardware::HWController;
use embsys::drivers::hardware::WIFIController;
use embsys::helpers;
use embsys::setup::SysInit;

use embassy_executor::SendSpawner;
use embassy_executor::Spawner;

use helpers::task_handler::Task;

#[embassy_executor::main]
async fn entry(spawner: Spawner) {
    defmt::info!("Initializing System..");
    SysInit::hardware_controller();

    defmt::info!("Initializing WiFi Driver..");
    SysInit::wifi_controller(&spawner).await;

    WIFIController::control_mut().gpio_set(0, true).await;

    let send_spawner: SendSpawner = spawner.make_send();
    let wifi_task: Task = Task::new(send_spawner, wifi_connect);

    WIFIController::control_mut().gpio_set(0, false).await;

    defmt::info!("Initializing Startup Tasks..");
    let _ = wifi_task.start();
    wifi_task.wait().await;

    let config_v4 = WIFIController::borrow_mut().get_config_v4();
    if let Some(config_v4) = config_v4 {
        let address = config_v4.address;
        defmt::info!("ADDRESS: {:?}", address);
    }

    WIFIController::control_mut().gpio_set(0, true).await;

    let endpoint = ArchonEndpoint::new(None, 9688);
    let mut archon = ArchonTransmitter::new(endpoint);
    let _ = archon.run().await;
}
