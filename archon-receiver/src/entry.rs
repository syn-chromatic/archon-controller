#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::receiver::ArchonReceiver;
use crate::tasks::archon_listen;
use crate::tasks::wifi_connect;
use crate::tasks::wifi_connect_static;

use archon_core::discovery::DiscoveryStatus;
use archon_core::discovery::EstablishInformation;
use archon_core::discovery::MultiCastDiscovery;
use archon_core::endpoint::ArchonEndpoint;
use archon_core::input::InputType;

use embsys::crates::cortex_m_rt;
use embsys::crates::defmt;
use embsys::crates::embassy_executor;
use embsys::crates::embassy_futures;
use embsys::drivers::hardware::WIFIController;
use embsys::exts::non_std;
use embsys::helpers;
use embsys::setup::SysInit;

use non_std::error::net::SocketError;

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

    let config_v4 = WIFIController::borrow_mut().get_config_v4();
    if let Some(config_v4) = config_v4 {
        let address = config_v4.address;
        defmt::info!("ADDRESS: {:?}", address);
    }

    WIFIController::control_mut().gpio_set(0, true).await;

    let discovery: MultiCastDiscovery = MultiCastDiscovery::new();
    let result: Result<EstablishInformation, SocketError> = discovery.announce().await;
    if let Ok(establish) = result {
        defmt::info!("Establish: {:?}", establish);
        let endpoint: ArchonEndpoint = establish.archon_endpoint();
        ArchonReceiver::read_lock().set_endpoint(endpoint);

        defmt::info!("Archon is in listening mode..");
        let archon_listen_task: Task = Task::new(send_spawner, archon_listen);
        let _ = archon_listen_task.start();

        loop {
            embassy_futures::yield_now().await;
            let input_type: Option<InputType> = ArchonReceiver::read_lock().take();
            if let Some(input_type) = input_type {
                input_type.defmt();
            }
        }
    } else if let Err(error) = result {
        defmt::info!("Error: {:?}", error);
    }
}
