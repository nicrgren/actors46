use actors46::{Actor, Handler, Message};
use async_trait::async_trait;

#[derive(Default)]
struct Stringer(Vec<char>);

impl Actor for Stringer {}

struct Push(char);

impl Message for Push {
    type Response = ();
}

struct Flush;

impl Message for Flush {
    type Response = String;
}

#[async_trait]
impl Handler<Push> for Stringer {
    async fn handle(&mut self, request: Push) {
        self.0.push(request.0);
    }
}

#[async_trait]
impl Handler<Flush> for Stringer {
    async fn handle(&mut self, _: Flush) -> String {
        self.0.drain(0..).collect()
    }
}

#[tokio::test]
async fn stringer() {
    let mut handle = Stringer::default().start();
    let s = "Hello, this is a test";

    for c in s.chars() {
        handle.send(Push(c)).await.expect("Failed to push");
    }

    let res = handle.send(Flush).await.expect("Flushing String");

    assert_eq!(res, s);
}
