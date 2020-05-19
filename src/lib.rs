mod actor;
pub mod handle;
mod message;

pub use actor::{Actor, Handler};
pub use handle::Handle;
pub use message::Message;

use message::{AddressedMessage, Envelope};
