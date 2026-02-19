use std::collections::HashMap;

use kameo::Actor;
use kameo::message::{Context, Message};

#[derive(Actor)]
pub struct PackCacheActor {
    cache: HashMap<i32, bool>,
}

impl PackCacheActor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl Default for PackCacheActor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Query(pub i32);

pub struct Insert(pub i32);

pub struct Invalidate(pub i32);

pub struct Clear;

impl Message<Query> for PackCacheActor {
    type Reply = Option<bool>;

    async fn handle(&mut self, msg: Query, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.cache.get(&msg.0).copied()
    }
}

impl Message<Insert> for PackCacheActor {
    type Reply = ();

    async fn handle(&mut self, msg: Insert, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.cache.insert(msg.0, true);
    }
}

impl Message<Invalidate> for PackCacheActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: Invalidate,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.cache.remove(&msg.0);
    }
}

impl Message<Clear> for PackCacheActor {
    type Reply = ();

    async fn handle(&mut self, _msg: Clear, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.cache.clear();
    }
}
