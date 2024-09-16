pub(crate) mod implementation {
    #[macro_export]
    macro_rules! node_sub_enum {
        ($(#[$config:meta])* $vis:vis enum $wrapper:ident {
            $($name:ident($modifier:tt $($t:tt)?)$(,)?)*
        }) => {$crate::paste! {
            #[derive($crate::RefCast)]
            #[repr(transparent)]
            $(
            #[$config]
            )*
            $vis struct $wrapper($crate::graph::AnyNode);

            $vis enum [<$wrapper Enum>]<'lt> {
                $(
                    $name(&'lt $crate::ty!($modifier $($t)?)),
                )*
            }

            $vis enum [<$wrapper EnumMut>]<'lt> {
                $(
                    $name(&'lt mut $crate::ty!($modifier $($t)?)),
                )*
            }

            impl $crate::graph::SubEnum for $wrapper {
                const VARIANTS: &'static [$crate::graph::AnyNodeD] =
                    $crate::concat_slices!([$crate::graph::AnyNodeD]: $($crate::node_sub_enum_entry!($modifier $($t)?), )*);
            }

            $(impl From<$crate::ty!($modifier $($t)?)> for $wrapper {
                #[inline]
                fn from(value: $crate::ty!($modifier $($t)?)) -> Self {
                    Self(value.into())
                }
            })*

            impl From<$wrapper> for $crate::graph::AnyNode {
                #[inline]
                fn from(value: $wrapper) -> Self {
                    value.0
                }
            }
            
            impl<'a> TryFrom<&'a $crate::graph::AnyNode> for &'a $wrapper {
                type Error = $crate::utils::Skip<std::convert::Infallible>;
                #[inline]
                fn try_from(value: &'a $crate::graph::AnyNode) -> Result<Self, Self::Error> {
                    if <$wrapper as $crate::graph::SubEnum>::contains(value) {
                        Ok(<$wrapper as $crate::RefCast>::ref_cast(value))
                    } else {
                        Err($crate::utils::Skip::Skipped)
                    }
                }
            }
            
            impl<'a> TryFrom<&'a mut $crate::graph::AnyNode> for &'a mut $wrapper {
                type Error = $crate::utils::Skip<std::convert::Infallible>;
                #[inline]
                fn try_from(value: &'a mut $crate::graph::AnyNode) -> Result<Self, Self::Error> {
                    if <$wrapper as $crate::graph::SubEnum>::contains(value) {
                        Ok(<$wrapper as $crate::RefCast>::ref_cast_mut(value))
                    } else {
                        Err($crate::utils::Skip::Skipped)
                    }
                }
            }

            impl $crate::traits::Identifiable for $wrapper {
                #[inline]
                fn get_id(&self) -> $crate::graph::NodeId<Self> {
                    self.0.get_id().coerce()
                }
            }
            
            impl<'lt> $crate::traits::AsEnum for &'lt $wrapper {
                type Enum = [<$wrapper Enum>]<'lt>;
                
                #[inline]
                fn as_enum(self) -> Self::Enum {
                    $(
                        $crate::node_sub_enum_match_entry!(self.0 => $name, $modifier $($t)?);
                    )*
                    unreachable!()
                }
            }
            
            impl<'lt> $crate::traits::AsEnum for &'lt mut $wrapper {
                type Enum = [<$wrapper EnumMut>]<'lt>;
                
                #[inline]
                fn as_enum(self) -> Self::Enum {
                    $(
                        $crate::node_sub_enum_match_entry!(self.0 => mut $name, $modifier $($t)?);
                    )*
                    unreachable!()
                }
            }
        }};
    }
    
    #[macro_export]
    macro_rules! node_sub_enum_match_entry {
        ($this:expr => $name:ident, $t:ident) => { 
            if let $crate::graph::AnyNode::$t(ref x) = $this {
                return Self::Enum::$name(x);
            } 
        };
        ($this:expr => mut $name:ident, $t:ident) => { 
            if let $crate::graph::AnyNode::$t(ref mut x) = $this {
                return Self::Enum::$name(x);
            } 
        };
        ($this:expr => $name:ident, forward $t:ty) => { 
            if <$t as $crate::graph::SubEnum>::contains(&$this) {
                return Self::Enum::$name(<$t as $crate::RefCast>::ref_cast(&$this))
            }
        };
        ($this:expr => mut $name:ident, forward $t:ty) => { 
            if <$t as $crate::graph::SubEnum>::contains(&$this) {
                return Self::Enum::$name(<$t as $crate::RefCast>::ref_cast_mut(&mut $this))
            }
        };
    }

    #[macro_export]
    macro_rules! ty {
        ($t:ty) => { $t };
        (forward $t:ty) => { $t };
    }

    #[macro_export]
    macro_rules! node_sub_enum_entry {
        ($t:ident) => { &[$crate::graph::AnyNodeD::$t] };
        (forward $t:ty) => { <$t as $crate::graph::SubEnum>::VARIANTS };
    }

    macro_rules! node {
    ($(#[$config:meta])* $vis:vis struct $name:ident {
        $($field_vis:vis $field_name:ident: $field_type:ty,)*;
        $($graph_vis:vis $graph_name:ident: $graph_type:ty$( as $tag:tt)?,)*
    }) => {
        $(#[$config])*
        $vis struct $name {
            id: std::cell::Cell<$crate::graph::NodeId<$name>>,
            $($field_vis $field_name: $field_type,)*
        }
        
        impl $name {
            pub fn uninit($($field_name: $field_type,)*) -> $crate::Uninit<'static, Self> {
                $crate::Uninit::new(Self {
                    id: std::cell::Cell::new($crate::graph::NodeId::null()),
                    $(
                    $field_name,
                    )*
                })
            }

            pub fn into_uninit(self) -> $crate::Uninit<'static, Self> {
                Self::uninit($(self.$field_name, )*)
            }
        }

        impl $crate::graph::Identifiable for $name {
            fn get_id(&self) -> $crate::graph::NodeId<Self> {
                self.id.get()
            }

            fn set_id(&self, value: $crate::graph::NodeId<Self>) {
                self.id.set(value)
            }
        }
        
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut fmt = f.debug_struct(stringify!($name));
                fmt.field("id", &self.id.get());
                $(
                    fmt.field(stringify!($field_name), &self.$field_name);
                )*
                fmt.finish()
            }
        }
        
        impl std::cmp::PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }
        
        impl $crate::graph::SubEnum for $name {
            const VARIANTS: &'static [$crate::graph::AnyNodeD] = &[$crate::graph::AnyNodeD::$name];
            
            #[inline(always)]
            fn contains(node: &$crate::graph::AnyNode) -> bool {
                node.describe() == $crate::graph::AnyNodeD::$name
            }
        }
        
        $crate::with_children! [$name => {
            $($graph_vis $graph_name: $graph_type $( as $tag)?,)*
        }];
    };
    ($(#[$config:meta])* $vis:vis struct $name:ident;) => {
        node!($(#[$config])* $vis struct $name {;});
    }
}

    pub(crate) use node;
}
