use crate::wrapper::InteractionWrapper;
use kodept_ast::prelude::NodeId;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_core::structure::Located;
use kodept_report::error::report::IntoSpannedReportMessage;
use kodept_report::error::traits::SpannedError;

mod lint;
mod normalize;
mod report;
mod scope;

pub mod prelude {
    pub use super::lint::LintDescriptor;
    pub use super::lint::{module::SingleModuleWithBrackets, rlt_linking::RLTLinkLint};

    pub use super::scope::builder::ScopeBuilder;
    pub use super::scope::symbol::{Symbol, SymbolKind, DuplicatedSymbolError, ExtractSymbols};

    pub use super::report::ASTExt;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Skip<E> {
    Skipped,
    Failed(E),
}

pub type Result<E> = std::result::Result<(), Skip<E>>;
type Ctx<'w> = kodept_ast::interaction::Interaction<'w>;

fn skip<E>() -> Result<E> {
    Err(Skip::Skipped)
}

fn fail<E>(error: E) -> Result<E> {
    Err(Skip::Failed(error))
}

fn done<E>() -> Result<E> {
    Ok(())
}

pub mod wrapper {
    use crate::report::Reporter;
    use crate::Skip;
    use bevy_ecs::prelude::{In, IntoSystem, IntoSystemConfigs};
    use bevy_ecs::schedule::SystemConfigs;
    use kodept_report::error::report::IntoSpannedReportMessage;
    use std::marker::PhantomData;
    use tracing::trace;

    pub struct InteractionWrapper<E>(PhantomData<E>, SystemConfigs);

    impl<E> InteractionWrapper<E> {
        pub(crate) fn wrap<M, S>(system: S) -> Self
        where
            S: IntoSystem<(), crate::Result<E>, M>,
            E: IntoSpannedReportMessage + 'static,
        {
            let name = system.system_type_id();
            let piped_system = system.pipe(
                move |In(result): In<crate::Result<E>>, reporter: Reporter| match result {
                    Ok(()) => trace!("System {name:?} completed"),
                    Err(Skip::Skipped) => trace!("System {name:?} skipped"),
                    Err(Skip::Failed(e)) => reporter.report(e),
                },
            );
            Self(PhantomData, IntoSystemConfigs::into_configs(piped_system))
        }

        pub(crate) fn from_configs(configs: impl IntoSystemConfigs<()>) -> Self {
            InteractionWrapper(PhantomData, configs.into_configs())
        }

        pub fn unwrap(self) -> impl IntoSystemConfigs<()> {
            self.1
        }
    }
}

pub trait Interaction<M = ()> {
    type Error: IntoSpannedReportMessage + 'static;

    fn interaction() -> InteractionWrapper<Self::Error>;

    fn install(ctx: &mut Ctx) {
        ctx.register(Self::interaction().unwrap());
    }
}

pub trait SpannedErrorExt<E> {
    fn for_node(inner: E, node_id: NodeId, syntax: &SyntaxResolver) -> Self;
}

impl<E: std::error::Error> SpannedErrorExt<E> for SpannedError<E> {
    fn for_node(inner: E, node_id: NodeId, syntax: &SyntaxResolver) -> Self {
        let variant = syntax.get_unknown(node_id).unwrap();
        let point = variant.location();
        Self::new(inner, point)
    }
}
