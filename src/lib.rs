mod actor;
mod message;

pub use actor::{Actor, Handle, Handler};
pub use message::Message;

use message::{AddressedMessage, Envelope};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deliver message to actor")]
    SendFailed,

    #[error("Receive failed. Actor was stopped")]
    ReceiveFailed,
}
