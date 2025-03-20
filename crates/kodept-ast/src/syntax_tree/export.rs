use crate::properties::Root;
use bevy_ecs::prelude::*;
use std::io::Write;

impl super::storage::AST {
    pub fn export_dot<W: Write + 'static>(&mut self, writer: &mut W) -> std::io::Result<()> {
        let root = self
            .world
            .query_filtered::<Entity, With<Root>>()
            .single(&self.world)
            .expect("Cannot get AST root");
        
        let components = self.world.inspect_entity(root).unwrap();
        write!(writer, "{:#?}", components.collect::<Vec<_>>())?;
        Ok(())
    }
}
