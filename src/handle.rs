use tokio::sync::mpsc;

use crate::{Actor, AddressedMessage, Envelope, Handler, Message};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deliver message to actor")]
    SendFailed,

    #[error("Receive failed. Actor was stopped")]
    ReceiveFailed,
}

pub struct Handle<A>
where
    A: Actor,
{
    si: mpsc::Sender<Box<dyn AddressedMessage<Actor = A>>>,
}

impl<A> Clone for Handle<A>
where
    A: Actor,
{
    fn clone(&self) -> Self {
        Self {
            si: self.si.clone(),
        }
    }
}

impl<A> Handle<A>
where
    A: Actor,
{
    pub(crate) fn new(si: mpsc::Sender<Box<dyn AddressedMessage<Actor = A>>>) -> Self {
        Self { si }
    }

    pub async fn send<M>(&mut self, message: M) -> Result<M::Response, Error>
    where
        M: Message,
        A: Handler<M>,
    {
        let (envelope, res_fut) = Envelope::wrap_for_response(message);
        if let Err(_err) = self.si.send(Box::new(envelope)).await {
            return Err(Error::SendFailed);
        }

        match res_fut.await {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::ReceiveFailed),
        }
    }
}
