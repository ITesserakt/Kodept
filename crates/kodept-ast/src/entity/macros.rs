#[macro_export]
macro_rules! define_union {
    ($vis:vis enum $this:ident[$this_item:ident$(, $this_filter:ident)?] {
        $($variant:ident$(|)?)+
    }) => {
        #[derive(Clone, Copy)]
        $vis struct $this<'a> {
            pub id: $crate::prelude::NodeId,
            inner: $this_item<'a>
        }

        #[derive(Clone, Copy)]
        $vis enum $this_item<'a> {
            $(
            $variant($crate::prelude::NodeRef<'a, &'a $variant>),
            )+
        }

        $crate::define_filter!($vis, $($this_filter)? = $($variant)+);

        impl<'a> $crate::prelude::FromEnum<'a> for $this<'a> {
            #[inline(always)]
            fn from_enum(value: $crate::prelude::AnyNodeRefItem<'a, '_>) -> Option<Self> {
                let id = value.id();
                let inner = None
                    $(.or_else(|| value.get().map($this_item::$variant)))+?;
                Some($this { id, inner })
            }
        }

        impl<'a> std::ops::Deref for $this<'a> {
            type Target = $this_item<'a>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };
}

#[macro_export]
macro_rules! define_filter {
    ($vis:vis, = $($variant:ident)+) => {};
    ($vis:vis, $this_filter:ident = $($variant:ident)+) => {
        type $this_filter = (
            bevy_ecs::prelude::With<$crate::properties::Node>,
            bevy_ecs::prelude::Or<($(
                bevy_ecs::prelude::With<$variant>,
            )+)>
        );
    }
}
