use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::{Actor, Handler};

pub trait Message
where
    Self: Send + 'static,
{
    type Response: 'static + Send;
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
    // This two are optional in order to be able to use take on a &mut ref.
    // If not, <Envelope as AddressedMessage> would need to be taken by
    // value, which is not possible.
    message: Option<M>,
    res_si: Option<oneshot::Sender<M::Response>>,

    _marker: std::marker::PhantomData<A>,
}

impl<A, M> Envelope<A, M>
where
    A: Actor,
    M: Message,
    A: Handler<M>,
{
    pub(crate) fn wrap(message: M) -> (Self, oneshot::Receiver<M::Response>) {
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
        let res_si = self.res_si.take().unwrap();

        if let Err(_) = res_si.send(response) {
            log::error!("Could not return response. Dropping");
        }
    }
}
