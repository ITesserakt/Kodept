use nonempty_collections::nev;

use kodept_ast::{Type, TypeEnum};
use kodept_ast::graph::{PermTkn, SyntaxTree};
use kodept_ast::traits::AsEnum;
use kodept_inference::algorithm_w::AlgorithmWError;
use kodept_inference::language::var;
use kodept_inference::r#type::{MonomorphicType, Tuple};

use crate::scope::ScopeSearch;
use crate::type_checker::InferError;

pub(crate) fn convert(
    ty: &Type,
    scope: ScopeSearch,
    ast: &SyntaxTree,
    token: &PermTkn,
) -> Result<MonomorphicType, InferError> {
    return match ty.as_enum() {
        TypeEnum::TyName(constant) => {
            scope
                .ty(&constant.name)
                .ok_or(InferError::AlgorithmW(AlgorithmWError::UnknownVar(nev![
                    var(&constant.name)
                ])))
        }
        TypeEnum::Tuple(tuple) => {
            let types: Result<Vec<_>, _> = tuple
                .types(ast, token)
                .into_iter()
                .map(|it| match convert(it, scope.clone(), ast, token) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(e),
                })
                .collect();
            let types = types?;
            Ok(MonomorphicType::Tuple(Tuple::new(types)).into())
        }
    };
}
