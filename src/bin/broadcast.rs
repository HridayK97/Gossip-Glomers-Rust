use async_trait::async_trait;
use chrono::Utc;
use maelstrom::protocol::Message;
use maelstrom::{done, Node, Result, Runtime};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_context::context::Context;

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
                if !self.check_message_exists(&message).await {
                    self.insert_message(message).await;
                    let neighbors: Vec<String> =
                        runtime.neighbours().map(|x| x.to_owned()).collect();
                    for neighbor in neighbors.into_iter() {
                        let runtime = runtime.clone();
                        tokio::task::spawn(async move {
                            gossip_offer_to_node(runtime.clone(), neighbor, message).await;
                        });
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

impl Handler {
    async fn check_message_exists(&self, message: &i64) -> bool {
        let store = self.store.read().await;
        store.contains(message)
    }

    async fn insert_message(&self, message: i64) {
        let mut store = self.store.write().await;
        store.insert(message);
    }
}

async fn gossip_offer_to_node(runtime: Runtime, node: String, message: i64) {
    while !runtime
        .call(
            // This just hangs on the await during a network partition if you don't pass a timeout!!
            // Passing a timeout of 100 ms
            Context::with_timeout(Duration::from_millis(100)).0,
            node.clone(),
            Request::Broadcast { message },
        )
        .await
        .is_ok()
    {
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

fn debug(message: String) {
    eprintln!(
        "{:?} - {:?}",
        message,
        Utc::now().to_rfc3339()
    );
}
