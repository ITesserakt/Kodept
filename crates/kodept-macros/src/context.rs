use crate::error::report::{IntoSpannedReportMessage, Report};
use crate::error::report_collector::ReportCollector;
use kodept_ast::graph::tags::ChildTag;
use kodept_ast::graph::{
    AnyNode, AnyNodeD, AnyNodeId, HasChildrenMarker, Identifiable, NodeId, SyntaxTree,
};
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_ast::Uninit;
use kodept_core::file_name::FileName;
use kodept_core::Freeze;
use std::mem::replace;
use kodept_ast::graph::node_props::Node;

pub type FileId = u16;

#[derive(Debug)]
pub struct FileDescriptor {
    pub name: FileName,
    pub id: FileId,
}

#[derive(Debug)]
pub struct Context<'r> {
    pub ast: SyntaxTree,
    pub rlt: RLTAccessor<'r>,
    pub collector: &'r mut ReportCollector,
    pub current_file: Freeze<FileDescriptor>,
}

impl<'rlt> Context<'rlt> {
    pub fn describe(&self, node_id: AnyNodeId) -> AnyNodeD {
        self.ast
            .get(node_id)
            .expect("Cannot find node with given id")
            .describe()
    }

    pub fn report_and_fail<T>(
        &mut self,
        message: impl IntoSpannedReportMessage,
    ) -> Result<T, Report<FileId>> {
        Err(Report::from_message(self.current_file.id, message))
    }

    pub fn report(&mut self, message: impl IntoSpannedReportMessage) {
        self.collector.report(self.current_file.id, message)
    }

    #[allow(unsafe_code)]
    pub fn replace<T>(&mut self, node_id: NodeId<T>, value: Uninit<'rlt, T>) -> Option<Uninit<T>>
    where
        T: TryFrom<AnyNode>,
        AnyNode: From<T>,
    {
        // SAFETY: rlt was already linked with that node id
        let (value, rlt) = unsafe { value.map_into().unwrap_unchecked(node_id.widen()) };
        // Relink if updated
        if let Some(replaced) = rlt {
            self.rlt.set(node_id, replaced);
        }

        let slot = self.ast.get_mut(node_id.widen())?;
        let old = replace(slot, value);
        old.set_id(AnyNodeId::null());

        Some(Uninit::new(old.try_into().ok()?))
    }

    pub fn add_child<T, U, const TAG: ChildTag>(
        &mut self,
        parent_id: NodeId<T>,
        value: Uninit<'rlt, U>,
    ) -> NodeId<U>
    where
        T: HasChildrenMarker<U, TAG>,
        U: Identifiable + Node,
        AnyNode: From<U>,
    {
        let mut rlt = None;
        let id = self.ast.add_child(
            parent_id,
            |id| {
                let (value, link) = value.unwrap(id);
                rlt = link;
                value
            },
            TAG,
        );
        if let Some(rlt) = rlt {
            self.rlt.set(id, rlt);
        }
        id
    }
}
