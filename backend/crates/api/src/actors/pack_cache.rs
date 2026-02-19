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

pub struct Query {
    pub pack_version_id: i32,
}

pub struct Insert {
    pub pack_version_id: i32,
    pub is_verified: bool,
}

pub struct Invalidate {
    pub pack_version_id: i32,
}

pub struct Clear;

impl Message<Query> for PackCacheActor {
    type Reply = Option<bool>;

    async fn handle(&mut self, msg: Query, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.cache.get(&msg.pack_version_id).copied()
    }
}

impl Message<Insert> for PackCacheActor {
    type Reply = ();

    async fn handle(&mut self, msg: Insert, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.cache.insert(msg.pack_version_id, msg.is_verified);
    }
}

impl Message<Invalidate> for PackCacheActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: Invalidate,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.cache.remove(&msg.pack_version_id);
    }
}

impl Message<Clear> for PackCacheActor {
    type Reply = ();

    async fn handle(&mut self, _msg: Clear, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.cache.clear();
    }
}
