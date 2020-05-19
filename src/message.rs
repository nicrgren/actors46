use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::{Actor, Handler};

pub trait Message
where
    Self: Send + 'static,
{
    type Response: 'static + Send;
}

pub(crate) trait IntoAddressed
where
    Self: Message + Sized,
{
    fn address_to<A>(self) -> Box<dyn AddressedMessage<Actor = A>>
    where
        A: Actor,
        A: Handler<Self>;
}

impl<M> IntoAddressed for M
where
    M: Message<Response = ()> + Sized,
{
    fn address_to<A>(self) -> Box<dyn AddressedMessage<Actor = A>>
    where
        A: Actor,
        A: Handler<M>,
    {
        Box::new(Envelope::wrap(self))
    }
}

#[async_trait]
pub(crate) trait AddressedMessage
where
    Self: 'static + Send,
{
    type Actor: Actor;

    async fn handle(&mut self, actor: &mut Self::Actor);
}

pub(crate) struct Envelope<A, M>
where
    A: Actor,
    M: Message,
    A: Handler<M>,
{
    // This message is optional in order to be able to use take on a &mut ref.
    // If not, <Envelope as AddressedMessage> would need to be taken by
    // value, which is not possible.
    message: Option<M>,

    // This sink is None when M::Response = ();
    res_si: Option<oneshot::Sender<M::Response>>,

    _marker: std::marker::PhantomData<A>,
}

impl<A, M> Envelope<A, M>
where
    A: Actor,
    M: Message,
    A: Handler<M>,
{
    /// Wraps a message together with a response channel that will deliver the message reply.
    pub(crate) fn wrap_for_response(message: M) -> (Self, oneshot::Receiver<M::Response>) {
        let (res_si, res_st) = oneshot::channel();
        (
            Self {
                message: Some(message),
                res_si: Some(res_si),
                _marker: std::marker::PhantomData,
            },
            res_st,
        )
    }
}
impl<A, M> Envelope<A, M>
where
    A: Actor,
    M: Message<Response = ()>,
    A: Handler<M>,
{
    /// Wraps a message to be delivered to the Actor. Without expecting a reply.
    pub(crate) fn wrap(message: M) -> Self {
        Self {
            message: Some(message),
            res_si: None,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<A, M> AddressedMessage for Envelope<A, M>
where
    A: Actor,
    M: Message,
    A: Handler<M>,
{
    type Actor = A;

    async fn handle(&mut self, actor: &mut Self::Actor) {
        let response = actor.handle(self.message.take().unwrap()).await;

        if let Some(si) = self.res_si.take() {
            if si.send(response).is_err() {
                log::error!("Could not return response. Dropping");
            }
        }
    }
}
