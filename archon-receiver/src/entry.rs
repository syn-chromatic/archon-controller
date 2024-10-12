#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::receiver::ArchonReceiver;
use crate::tasks::archon_init;
use crate::tasks::archon_listen;
use crate::tasks::wifi_connect;

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
async fn rp2040_entry(spawner: Spawner) {
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

    defmt::info!("Initializing Archon..");
    let archon_init_task: Task = Task::new(send_spawner, archon_init);
    let _ = archon_init_task.start();
    let _ = archon_init_task.wait().await;

    defmt::info!("Archon is in listening mode..");
    let archon_listen_task: Task = Task::new(send_spawner, archon_listen);
    let _ = archon_listen_task.start();

    WIFIController::control_mut().gpio_set(0, true).await;

    loop {
        embassy_futures::yield_now().await;
        let input_type: Option<InputType> = ArchonReceiver::read_lock().take();
        if let Some(input_type) = input_type {
            input_type.defmt();
        }
    }
}
