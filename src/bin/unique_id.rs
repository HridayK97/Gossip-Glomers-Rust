use async_trait::async_trait;
use maelstrom::protocol::Message;
use maelstrom::{done, Node, Result, Runtime};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) fn main() -> Result<()> {
    Runtime::init(try_main())
}

async fn try_main() -> Result<()> {
    let handler = Arc::new(Handler::default());
    Runtime::new().with_handler(handler).run().await
}

#[derive(Clone, Default)]
struct Handler {}

#[async_trait]
impl Node for Handler {
    async fn process(&self, runtime: Runtime, req: Message) -> Result<()> {
        if req.get_type() == "generate" {
            let mut response = req.body.clone().with_type("generate_ok");


            response.extra.insert("id".to_string(), Uuid::new_v4().to_string().into());



            // let my_uuid = Uuid::new_v4();
            // println!("{}", my_uuid);

            return runtime.reply(req, response).await;
        }

        done(runtime, req)
    }
}