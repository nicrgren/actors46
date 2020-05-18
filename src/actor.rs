use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{AddressedMessage, Envelope, Error, Message};

pub trait Actor
where
    Self: Sized + Send + 'static,
{
    fn start(self) -> Handle<Self> {
        Context::run(self)
    }

    // @TODO: Add this to enforce starting on specified System.
    // fn start_on(self);
}

#[async_trait]
pub trait Handler<M>
where
    M: Message,
{
    async fn handle(&mut self, message: M) -> M::Response;
}

/// ActorContext takes care of running an Actor.
/// It manages the actor loop and all it's incoming message streams.
struct Context<A>
where
    A: Actor,
{
    message_st: mpsc::Receiver<Box<dyn AddressedMessage<Actor = A>>>,
}

pub struct Handle<A>
where
    A: Actor,
{
    chan: mpsc::Sender<Box<dyn AddressedMessage<Actor = A>>>,
}

impl<A> Handle<A>
where
    A: Actor,
{
    pub async fn send<M>(&mut self, message: M) -> Result<M::Response, Error>
    where
        M: Message,
        A: Handler<M>,
    {
        let (envelope, res_fut) = Envelope::wrap(message);
        if let Err(_err) = self.chan.send(Box::new(envelope)).await {
            return Err(Error::SendFailed);
        }

        match res_fut.await {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::ReceiveFailed),
        }
    }
}

impl<A> Context<A>
where
    A: Actor,
{
    fn run(actor: A) -> Handle<A> {
        let (si, st) = mpsc::channel(1);

        let context = Context { message_st: st };
        let handle = Handle { chan: si };

        // @TODO: This join handle could be stored within the ActorSystem.
        // ActorSystem is not yet added.
        let _join_handle = tokio::spawn(context.main_loop(actor));

        handle
    }

    async fn main_loop(mut self, mut actor: A) {
        while let Some(ref mut envelope) = self.message_st.recv().await {
            envelope.handle(&mut actor).await;
        }
    }
}
