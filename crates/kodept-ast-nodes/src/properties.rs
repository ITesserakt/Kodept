use kodept_ast::external::Component;
use kodept_ast::properties::NodeProperty;
use kodept_ast::properties::tags::tags::Tagged;

#[derive(Debug, Component, Default)]
pub struct TopLevel;

impl NodeProperty for TopLevel {}
impl Tagged for TopLevel {}

#[derive(Debug, Component, Default)]
pub struct Type;

impl NodeProperty for Type {}
impl Tagged for Type {}

#[derive(Debug, Component, Default)]
pub struct Param;

impl NodeProperty for Param {}
impl Tagged for Param {}