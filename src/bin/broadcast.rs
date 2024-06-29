use async_trait::async_trait;
use maelstrom::protocol::Message;
use maelstrom::{done, Node, Result, Runtime};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) fn main() -> Result<()> {
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::default());
    Runtime::new().with_handler(handler).run().await
}

#[derive(Clone, Default)]
struct Handler {
    store: Arc<RwLock<HashSet<i64>>>,
}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        let request_body: Result<Request> = req.body.as_obj();

        match request_body {
            Ok(Request::Broadcast { message }) => {
                let mut store = self.store.write().await;
                if !store.contains(&message) {
                    store.insert(message);

                    for neighbor in runtime.neighbours() {
                        runtime.call_async(neighbor, Request::Broadcast { message })
                    }
                }
                return runtime.reply_ok(req).await;
            }

            Ok(Request::Read {}) => {
                let store = self.store.read().await;
                let data: Vec<i64> = store.clone().into_iter().collect();
                let msg = Request::ReadOk { messages: data };
                return runtime.reply(req, msg).await;
            }

            Ok(Request::Topology { topology }) => {
                return runtime.reply_ok(req).await;
            }

            _ => done(runtime, req),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Request {
    Init {},
    Read {},
    ReadOk {
        messages: Vec<i64>,
    },
    Broadcast {
        message: i64,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
}
