#[macro_export]
macro_rules! define_phase {
    ($vis:vis phase $name:ident[$label:ident]
        {$($field_vis:vis $field:ident: $ty:ty$(,)?)*}
        fn build($self:ident, $engine:ident: $engine_ty:ty) $build:block
    ) => {
        #[derive(Debug, bevy_ecs::prelude::SystemSet, Clone, Copy, PartialEq, Eq, Hash, Default)]
        $vis struct $label;

        $vis struct $name {
            $(
            $field_vis $field: $ty
            )*
        }

        impl $crate::engine::Phase for $name {
            type Set = $label;

            fn build($self, $engine: $engine_ty) {
                $build
            }
        }
    };
    (
        $vis:vis phase $name:ident[$label:ident];
        fn build($self:ident, $engine:ident: $engine_ty:ty) $build:block
    ) => {
        #[derive(Debug, bevy_ecs::prelude::SystemSet, Clone, Copy, PartialEq, Eq, Hash, Default)]
        $vis struct $label;
        $vis struct $name;

        impl $crate::engine::Phase for $name {
            type Set = $label;

            fn build($self, $engine: $engine_ty) {
                $build
            }
        }
    }
}
