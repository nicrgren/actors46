use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{AddressedMessage, Handle, Message};

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

impl<A> Context<A>
where
    A: Actor,
{
    fn run(actor: A) -> Handle<A> {
        let (si, st) = mpsc::channel(1);

        let context = Context { message_st: st };
        let handle = Handle::new(si);

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
