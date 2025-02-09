use kodept_ast::graph::{AnyNode, AnyNodeD, AnyNodeId, HasChildrenMarker, Identifiable, NodeId, SyntaxTree};
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_core::file_name::{FileDescriptor};
use kodept_core::Freeze;
use kodept_report::prelude::{ad_hoc_message, IntoSpannedReportMessage, Report, SpannedReportMessage};
use std::fmt::{Debug, Formatter};
use std::mem::replace;
use kodept_ast::graph::node_props::Node;
use kodept_ast::graph::tags::ChildTag;
use kodept_ast::Uninit;

pub struct Context<'r> {
    pub ast: SyntaxTree,
    pub rlt: RLTAccessor<'r>,
    report_sink: Box<dyn Fn(Report)>,
    current_file: Freeze<FileDescriptor>,
}

impl<'a> Context<'a> {
    pub fn new(
        ast: SyntaxTree,
        rlt: RLTAccessor<'a>,
        report_sink: impl Fn(Report) + 'static,
        current_file: FileDescriptor,
    ) -> Self {
        Self {
            ast,
            rlt,
            report_sink: Box::new(report_sink),
            current_file: Freeze::new(current_file),
        }
    }

    pub fn current_file(&self) -> &FileDescriptor {
        &*self.current_file
    }
    
    pub fn describe(&self, node_id: AnyNodeId) -> AnyNodeD {
        self.ast
            .get(node_id)
            .expect("Cannot find node with given id")
            .describe()
    }

    pub fn report(&self, message: impl IntoSpannedReportMessage) {
        let report = Report::from_message(self.current_file.id(), message);
        self.push_report(report)
    }
    
    pub fn report_adhoc<M>(&self, f: impl FnOnce() -> M) where M: SpannedReportMessage {
        let report = Report::from_message(self.current_file.id(), ad_hoc_message(f));
        self.push_report(report)
    }
    
    pub fn push_report(&self, report: Report) {
        (self.report_sink)(report)
    }
    
    pub fn replace<T>(&mut self, id: NodeId<T>, value: Uninit<'a, T>) -> Option<Uninit<T>> 
    where 
        T: TryFrom<AnyNode>,
        AnyNode: From<T>
    {
        let (value, rlt) = value.map_into().unwrap(id.widen());
        // Relink if updated
        if let Some(replaced) = rlt {
            self.rlt.set(id, replaced);
        }
        
        let slot = self.ast.get_mut(id.widen())?;
        let old = replace(slot, value);
        old.set_id(AnyNodeId::null());
        
        Some(Uninit::new(old.try_into().ok()?))
    }
    
    pub fn add_child<T, U, const TAG: ChildTag>(&mut self, parent_id: NodeId<T>, value: Uninit<'a, U>) -> NodeId<U>
    where 
        T: HasChildrenMarker<U, TAG>,
        U: Identifiable + Node,
        AnyNode: From<U>
    {
        let mut rlt = None;
        let id = self.ast.add_child(parent_id, |id| {
            let (value, link) = value.unwrap(id);
            rlt = link;
            value
        }, TAG);
        if let Some(rlt) = rlt {
            self.rlt.set(id, rlt);
        }
        id
    }
}

impl Debug for Context<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("ast", &self.ast)
            .field("rlt", &self.rlt)
            .field("current_file", &*self.current_file)
            .finish()
    }
}
