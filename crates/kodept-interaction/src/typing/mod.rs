use crate::report::Reporter;
use crate::utils::{Ctx, Disposable, Interaction, wrap_system};
use bevy_ecs::prelude::{Add, On, Query};
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, StaticSystemParam, SystemParam},
};
use kodept_ast::properties::SourceSpan;
use kodept_inference::r#type::MonomorphicType;
use kodept_report::prelude::{Diagnostic, Severity};

mod function;
mod literals;
mod type_refs;
mod refs;

pub struct TypeInferPass;

#[derive(Debug, Component)]
#[component(immutable)]
struct Typed(MonomorphicType);

impl Interaction for TypeInferPass {
    fn install(ctx: &mut Ctx) -> impl Disposable + use<> {
        ctx.register(wrap_system(Self::name(), literals::system));
        ctx.register(wrap_system(Self::name(), type_refs::ty_system));
        ctx.register(wrap_system(Self::name(), type_refs::prod_ty_system));
        ctx.register(wrap_system(Self::name(), refs::system));

        ctx.immediate_exclusive(|w| {
            w.add_observer(
                |type_added: On<Add, Typed>,
                 query: Query<(&SourceSpan, &Typed)>,
                 reporter: Reporter| {
                    let Ok((span, ty)) = query.get(type_added.entity) else {
                        unreachable!();
                    };
                    reporter.report_ad_hoc(|| {
                        Diagnostic::new(Severity::Note)
                            .with_message("Type resolved")
                            .with_primary_label(format!("{}", ty.0), span.0)
                    })
                },
            );
        });
    }
}

impl std::ops::Deref for Typed {
    type Target = MonomorphicType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
