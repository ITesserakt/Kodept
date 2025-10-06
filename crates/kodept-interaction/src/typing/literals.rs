use crate::typing::Typed;
use bevy_ecs::{
    entity::Entity,
    query::Without,
    system::{Commands, Populated},
};
use kodept_ast_nodes::literal::Literal;
use kodept_inference::r#type::{MonomorphicType, PrimitiveType};
use std::num::NonZeroU8;

#[allow(unsafe_code)]
const fn bit_length(value: i128) -> NonZeroU8 {
    if value == 0 {
        // SAFETY: 1 is greater that 0
        unsafe { NonZeroU8::new_unchecked(1) }
    } else if value < 0 {
        // SAFETY: n.leading_zeroes must return 128 in order to violate contract, but this case was already processed
        unsafe { NonZeroU8::new_unchecked(128 - (-value).leading_zeros() as u8) }
    } else {
        // SAFETY: n.leading_zeroes must return 128 in order to violate contract, but this case was already processed
        unsafe { NonZeroU8::new_unchecked(128 - value.leading_zeros() as u8) }
    }
}

pub(super) fn system(
    literals: Populated<(Entity, &Literal), Without<Typed>>,
    mut commands: Commands,
) {
    for (id, literal) in literals.iter() {
        let ty = match literal {
            Literal::Integer(value) => {
                if value.is_negative() {
                    PrimitiveType::I(bit_length(*value).saturating_add(1))
                } else {
                    PrimitiveType::U(bit_length(*value))
                }
            }
            Literal::Floating(_) => {
                // FIXME: add check if value is greater that f24
                PrimitiveType::f24()
            }
            Literal::Char(_) => PrimitiveType::u8(),
            Literal::String(value) => PrimitiveType::StaticString(value.len() as u64),
        };
        commands
            .entity(id)
            .insert(Typed(MonomorphicType::primitive(ty)));
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::bit_length;

    #[rstest]
    #[case(0, 1)]
    #[case(1, 1)]
    #[case(-1, 1)]
    #[case(37, 6)]
    #[case(-37, 6)]
    #[case(i128::MAX, 127)]
    fn test_signed_bit_length(#[case] input: i128, #[case] expected: u8) {
        assert_eq!(expected, bit_length(input).get())
    }
}
