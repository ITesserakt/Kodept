use crate::traits::Context;
use crate::transformer::Transformer;
use kodept_ast::visitor::visit_side::{VisitGuard, VisitSide};
use kodept_ast::visitor::TraversingResult;
use kodept_ast::{BlockLevel, ExpressionBlock};
use kodept_core::impl_named;
use std::convert::Infallible;
use std::mem::take;

#[derive(Debug)]
pub struct VariableScopeTransformer;

impl_named!(VariableScopeTransformer);

impl Transformer for VariableScopeTransformer {
    type Error = Infallible;
    type Node<'n> = &'n mut ExpressionBlock;

    fn transform<'n, 'c>(
        &self,
        guard: VisitGuard<Self::Node<'n>>,
        context: &mut impl Context<'c>,
    ) -> TraversingResult<Self::Error> {
        let node = guard.allow_only(VisitSide::Exiting)?;
        let items = take(&mut node.items());
        *node = Self::step(items, vec![], context, node);
        Ok(())
    }
}

impl VariableScopeTransformer {
    fn split_block<'c, C: Context<'c>>(
        block: Vec<BlockLevel>,
        index: usize,
        ctx: &mut C,
        node: &mut ExpressionBlock,
    ) -> Result<(Vec<BlockLevel>, ExpressionBlock), Vec<BlockLevel>> {
        let (inner_i, outer_i): (Vec<_>, _) =
            block.into_iter().enumerate().partition(|it| it.0 >= index);
        let (inner, outer) = (
            inner_i.into_iter().map(|it| it.1).collect::<Vec<_>>(),
            outer_i.into_iter().map(|it| it.1).collect::<Vec<_>>(),
        );

        if outer.is_empty() || inner.len() == 1 {
            return Err(inner);
        }

        Ok((
            outer,
            ExpressionBlock::instantiate(inner, ctx).link_with_existing(ctx, node),
        ))
    }

    fn step<'c, C: Context<'c>>(
        current_block: Vec<BlockLevel>,
        mut skips: Vec<usize>,
        ctx: &mut C,
        node: &mut ExpressionBlock,
    ) -> ExpressionBlock {
        let var_index = current_block
            .iter()
            .enumerate()
            .position(|(i, it)| matches!(it, BlockLevel::InitVar(_)) && !skips.contains(&i));

        let var_index = match var_index {
            None => {
                return ExpressionBlock::instantiate(current_block, ctx)
                    .link_with_existing(ctx, node)
            }
            Some(x) => x,
        };
        let split = Self::split_block(current_block, var_index, ctx, node);

        match split {
            Err(current_block) => {
                skips.push(var_index);
                Self::step(current_block, skips, ctx, node)
            }
            Ok((mut outer, inner_block)) => {
                let recurse = Self::step(inner_block.items, vec![], ctx, node);
                outer.push(BlockLevel::Block(recurse));
                ExpressionBlock::instantiate(outer, ctx).link_with_existing(ctx, node)
            }
        }
    }
}
