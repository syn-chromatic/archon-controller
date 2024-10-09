use crate::tasks::connect_wifi;
use crate::tasks::archon;

use embsys::crates::cortex_m_rt;
use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_rp;
use embsys::crates::embassy_time;
use embsys::drivers;
use embsys::helpers;
use embsys::setup::SysInit;

use embassy_executor::SendSpawner;
use embassy_executor::Spawner;
use embassy_rp::watchdog::Watchdog;
use embassy_time::Duration;

use drivers::hardware::HWController;
use helpers::task_handler::Task;

#[embassy_executor::main]
async fn rp2040_entry(spawner: Spawner) {
    defmt::info!("Initializing System..");
    SysInit::hardware_controller();

    let send_spawner: SendSpawner = spawner.make_send();
    let wifi_task: Task = Task::new(send_spawner, connect_wifi);

    defmt::info!("Initializing WiFi Driver..");
    SysInit::wifi_controller(&spawner).await;

    defmt::info!("Initializing Startup Tasks..");
    let _ = wifi_task.start();
    wifi_task.wait().await;

    let task = Task::new(send_spawner, archon);
    let _ = task.start();
    task.wait().await;

    // defmt::info!("Initializing Watchdog..");
    // let watchdog: &mut Watchdog = HWController::watchdog_mut();
    // watchdog.start(Duration::from_secs(8));
}
