use std::rc::Rc;

use slotmap::SecondaryMap;

use kodept_ast::graph::GenericNodeKey;
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::Language;
use kodept_inference::r#type::PolymorphicType;

pub struct Store {
    models_cache: SecondaryMap<GenericNodeKey, Rc<Language>>,
    types_cache: SecondaryMap<GenericNodeKey, Rc<PolymorphicType>>,
    constraints_cache: SecondaryMap<GenericNodeKey, Assumptions>,
}

impl Store {
    pub fn cache(&self) {}
}
