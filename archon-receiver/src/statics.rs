use crate::consts::INPUT_BUFFER;
use crate::receiver::ArchonReceiver;

use embsys::exts::non_std;
use non_std::cell::OpCell;

pub static mut RECEIVER: OpCell<ArchonReceiver<INPUT_BUFFER>> = OpCell::new();
