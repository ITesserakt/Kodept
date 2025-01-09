use crate::properties::NodeProperty;
use bevy_ecs::prelude::Component;

pub trait Tagged: Default + NodeProperty {}

#[derive(Debug, Component, Default)]
pub struct NoTag;
impl Tagged for NoTag {}
impl NodeProperty for NoTag {}
