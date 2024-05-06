use kodept_ast::graph::{GenericASTNode, NodeId};
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::Language;
use kodept_inference::r#type::PolymorphicType;
use std::collections::HashMap;
use std::rc::Rc;

type GNodeId = NodeId<GenericASTNode>;

pub struct Store {
    models_cache: HashMap<GNodeId, Rc<Language>>,
    types_cache: HashMap<GNodeId, Rc<PolymorphicType>>,
    constraints_cache: HashMap<GNodeId, Assumptions>,
}

impl Store {
    pub fn cache(&self) {}
}
