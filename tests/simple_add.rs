use actors46::{Actor, Handler, Message};
use async_trait::async_trait;

struct Squarer;

impl Actor for Squarer {}

struct Square(i64);

impl Message for Square {
    type Response = i64;
}

#[async_trait]
impl Handler<Square> for Squarer {
    async fn handle(&mut self, request: Square) -> i64 {
        request.0 * request.0
    }
}

#[tokio::test]
async fn simple_add() {
    let mut handle = Squarer.start();

    let res = handle
        .send(Square(2))
        .await
        .expect("Could not receive response");

    assert_eq!(res, 4);
}
