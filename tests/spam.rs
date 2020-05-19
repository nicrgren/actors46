use actors46::{Actor, Handler, Message};
use async_trait::async_trait;

#[derive(Default)]
struct Counter {
    count: usize,
}

impl Actor for Counter {}

struct Tick;

impl Message for Tick {
    type Response = ();
}

struct GetCount;

impl Message for GetCount {
    type Response = usize;
}

#[async_trait]
impl Handler<Tick> for Counter {
    async fn handle(&mut self, _: Tick) {
        self.count += 1;
    }
}

#[async_trait]
impl Handler<GetCount> for Counter {
    async fn handle(&mut self, _: GetCount) -> usize {
        self.count
    }
}

#[tokio::test]
async fn spam() {
    let no_tasks = 100;
    let msg_per_task = 100;
    let mut handle = Counter::default().start();

    let join_handles = (0..no_tasks)
        .map(|_| {
            let mut h = handle.clone();
            tokio::spawn(async move {
                for _ in 0..msg_per_task {
                    h.send(Tick).await.expect("Sending Tick");
                }
            })
        })
        .collect::<Vec<_>>();

    futures::future::join_all(join_handles).await;

    let count = handle.send(GetCount).await.expect("Sending GetCount");
    assert_eq!(count, no_tasks * msg_per_task);
}
