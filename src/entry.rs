use crate::configuration::ARCHON_RECEIVER;
use crate::controller::input::InputType;
use crate::controller::receiver::ArchonReceiver;
use crate::tasks::archon_listen;
use crate::tasks::connect_wifi;
use crate::tasks::initialize_archon;

use embsys::crates::cortex_m_rt;
use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::crates::embassy_time;
use embsys::exts::std;
use embsys::helpers;
use embsys::setup::SysInit;

use embassy_executor::SendSpawner;
use embassy_executor::Spawner;

use helpers::task_handler::Task;
use std::boxed::Box;

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

    defmt::info!("Initializing Archon..");
    let init_archon_task: Task = Task::new(send_spawner, initialize_archon);
    let _ = init_archon_task.start();
    let _ = init_archon_task.wait().await;

    defmt::info!("Archon is in listening mode..");
    let archon_listen_task: Task = Task::new(send_spawner, archon_listen);
    let _ = archon_listen_task.start();

    let archon: &mut Box<ArchonReceiver<32>> = unsafe { ARCHON_RECEIVER.get_mut() };

    loop {
        embassy_futures::yield_now().await;
        let input_type: Option<InputType> = archon.take();
        if let Some(input_type) = input_type {
            input_type.defmt();
        }

        embassy_time::Timer::after_secs(1).await;
    }
}
