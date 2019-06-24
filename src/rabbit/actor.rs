use actix::prelude::{Actor, Context};

struct ChatClient;

impl Actor for ChatClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {}
}
