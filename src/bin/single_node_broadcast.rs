use async_trait::async_trait;
use maelstrom::protocol::Message;
use maelstrom::{done, Node, Result, Runtime};
use serde_json::{Number, Value};
use std::collections::HashSet;
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
        if req.get_type() == "broadcast" {
            let message = req
                .body
                .extra
                .get("message")
                .expect("Expected a message value from broadcast");

            if let Value::Number(value) = message {
                let value = value
                    .as_i64()
                    .expect("Expected a valid number in the message");
                let mut store = self.store.write().await;
                if !store.contains(&value) {
                    store.insert(value);
                }
            }

            let mut response = req.body.clone().with_type("broadcast_ok");
            response.extra.clear();
            return runtime.reply(req, response).await;
        } else if req.get_type() == "read" {
            let mut response = req.body.clone().with_type("read_ok");
            let store = self.store.read().await;

            let json_messages: Vec<Value> = store
                .iter()
                .map(|x| Value::Number(Number::from(*x)))
                .collect();

            response
                .extra
                .insert("messages".to_string(), Value::Array(json_messages));
            return runtime.reply(req, response).await;
        } else if req.get_type() == "topology" {
            // Do not need to do anything with topology here

            let mut response = req.body.clone().with_type("topology_ok");
            // Maelstrom expects no topology key to be returned. I'm just cloning the request body and clearing the extra attributes added here.
            // We still need the reply_to and message ID in the response. Not the most neat way to do it, but it works.
            response.extra.clear();
            return runtime.reply(req, response).await;
        }

        done(runtime, req)
    }
}
