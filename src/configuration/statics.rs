use crate::controller::receiver::ArchonReceiver;

use embsys::exts::non_std;
use non_std::cell::OpCellBox;

pub static mut ARCHON_RECEIVER: OpCellBox<ArchonReceiver<32>> = OpCellBox::new();
