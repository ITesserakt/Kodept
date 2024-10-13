pub(crate) mod implementation {
    #[macro_export]
    macro_rules! node_sub_enum {
        ($(#[$config:meta])* $vis:vis enum $wrapper:ident {
            $($name:ident($modifier:tt $($t:tt)?)$(,)?)*
        }) => {
            #[derive($crate::RefCast)]
            #[repr(transparent)]
            $(
            #[$config]
            )*
            $vis struct $wrapper($crate::graph::AnyNode);

            $crate::paste! { $vis enum [<$wrapper Enum>]<'lt> {
                $(
                    $name(&'lt $crate::ty!($modifier $($t)?)),
                )*
            }}

            $crate::paste! { $vis enum [<$wrapper EnumMut>]<'lt> {
                $(
                    $name(&'lt mut $crate::ty!($modifier $($t)?)),
                )*
            }}

            impl $crate::graph::node_props::SubEnum for $wrapper {
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
            
            impl From<$crate::graph::AnyNode> for $wrapper {
                #[inline]
                fn from(value: $crate::graph::AnyNode) -> Self {
                    Self(value)
                }
            }

            impl $crate::graph::node_props::Node for $wrapper {
                #[inline]
                fn erase(self) -> $crate::graph::AnyNode {
                    self.0
                }

                #[inline]
                fn describe(&self) -> $crate::graph::AnyNodeD {
                    self.0.describe()
                }

                #[inline]
                fn try_from_mut(value: &mut $crate::graph::AnyNode) -> Result<&mut Self, $crate::graph::node_props::ConversionError> {
                    if <$wrapper as $crate::graph::node_props::SubEnum>::contains(value) {
                        Ok(<$wrapper as $crate::RefCast>::ref_cast_mut(value))
                    } else {
                        Err($crate::graph::node_props::ConversionError {
                            expected_types: <$wrapper as $crate::graph::node_props::SubEnum>::VARIANTS,
                            actual_type: value.describe()
                        })
                    }
                }

                fn try_from_ref(value: &$crate::graph::AnyNode) -> Result<&Self, $crate::graph::node_props::ConversionError> {
                    if <$wrapper as $crate::graph::node_props::SubEnum>::contains(value) {
                        Ok(<$wrapper as $crate::RefCast>::ref_cast(value))
                    } else {
                        Err($crate::graph::node_props::ConversionError {
                            expected_types: <$wrapper as $crate::graph::node_props::SubEnum>::VARIANTS,
                            actual_type: value.describe()
                        })
                    }
                }
            }

            impl $crate::graph::Identifiable for $wrapper {
                #[inline]
                fn get_id(&self) -> $crate::graph::NodeId<Self> {
                    self.0.get_id().coerce()
                }
                
                #[inline]
                fn set_id(&self, id: $crate::graph::NodeId<Self>) {
                    self.0.set_id(id.widen())
                }
            }
            
            impl<'lt> $crate::traits::AsEnum for &'lt $wrapper {
                type Enum = $crate::paste! { [<$wrapper Enum>]<'lt> };
                
                #[inline]
                fn as_enum(self) -> Self::Enum {
                    $(
                        $crate::node_sub_enum_match_entry!(self.0 => $name, $modifier $($t)?);
                    )*
                    unreachable!()
                }
            }
            
            impl<'lt> $crate::traits::AsEnum for &'lt mut $wrapper {
                type Enum = $crate::paste! { [<$wrapper EnumMut>]<'lt> };
                
                #[inline]
                fn as_enum(self) -> Self::Enum {
                    $(
                        $crate::node_sub_enum_match_entry!(self.0 => mut $name, $modifier $($t)?);
                    )*
                    unreachable!()
                }
            }
        };
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
            if <$t as $crate::graph::node_props::SubEnum>::contains(&$this) {
                return Self::Enum::$name(<$t as $crate::RefCast>::ref_cast(&$this))
            }
        };
        ($this:expr => mut $name:ident, forward $t:ty) => { 
            if <$t as $crate::graph::node_props::SubEnum>::contains(&$this) {
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
        (forward $t:ty) => { <$t as $crate::graph::node_props::SubEnum>::VARIANTS };
    }

    macro_rules! node {
    ($(#[$config:meta])* $vis:vis struct $name:ident {
        $($field_vis:vis $field_name:ident: $field_type:ty,)*;
        $($graph_vis:vis $graph_name:ident: $graph_type:ty$( as $tag:tt)?,)*
        $(; parent is [$($parent:tt)+])?
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

        impl $crate::graph::node_props::Node for $name {
            #[inline]
            fn describe(&self) -> $crate::graph::AnyNodeD {
                $crate::graph::AnyNodeD::$name
            }

            #[inline]
            fn erase(self) -> $crate::graph::AnyNode {
                $crate::graph::AnyNode::$name(self)
            }

            #[inline]
            fn try_from_ref(value: &$crate::graph::AnyNode) -> Result<&Self, $crate::graph::node_props::ConversionError> {
                match value {
                    $crate::graph::AnyNode::$name(x) => Ok(x),
                    _ => Err($crate::graph::node_props::ConversionError {
                        actual_type: value.describe(),
                        expected_types: <$name as $crate::graph::node_props::SubEnum>::VARIANTS
                    })
                }
            }

            #[inline]
            fn try_from_mut(value: &mut $crate::graph::AnyNode) -> Result<&mut Self, $crate::graph::node_props::ConversionError> {
                match value {
                    $crate::graph::AnyNode::$name(x) => Ok(x),
                    _ => Err($crate::graph::node_props::ConversionError {
                        actual_type: value.describe(),
                        expected_types: <$name as $crate::graph::node_props::SubEnum>::VARIANTS
                    })
                }
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
        
        impl $crate::graph::node_props::SubEnum for $name {
            const VARIANTS: &'static [$crate::graph::AnyNodeD] = &[$crate::graph::AnyNodeD::$name];
            
            #[inline(always)]
            fn contains(node: &$crate::graph::AnyNode) -> bool {
                node.describe() == $crate::graph::AnyNodeD::$name
            }
        }

        $($crate::parent_definition! { $name => [$($parent)+] })?

        $crate::with_children! [$name => {
            $($graph_vis $graph_name: $graph_type $( as $tag)?,)*
        }];
    };
    ($(#[$config:meta])* $vis:vis struct $name:ident;) => {
        node!($(#[$config])* $vis struct $name {;});
    }
}

    macro_rules! parent_definition {
        ($name:ident => [$parent:ty]) => {
            impl $crate::graph::node_props::HasParent for $name {
                type Parent = $parent;

                #[inline]
                fn parent<'a>(&self, ast: &'a $crate::graph::SyntaxTree) -> Option<&'a $parent> {
                    let id = <Self as $crate::graph::Identifiable>::get_id(self);
                    let node = ast.parent_of(id)?;
                    Some(<$parent as $crate::graph::node_props::Node>::try_from_ref(node)
                        .expect("Parent node type mismatch"))
                }
            }
        };
        ($name:ident => [$($parent:ident$(,)?)+]) => {
            $crate::paste! { $crate::node_sub_enum! {
                pub enum [<$name Parent>] {
                    $($parent($parent),)+
                }
            }}

            impl $crate::graph::node_props::HasParent for $name {
                type Parent = $crate::paste! { [<$name Parent>] };

                fn parent<'a>(&self, ast: &'a $crate::graph::SyntaxTree) -> Option<&'a Self::Parent> {
                    let id = <Self as $crate::graph::Identifiable>::get_id(self);
                    let node = ast.parent_of(id)?;
                    Some(<Self::Parent as $crate::graph::node_props::Node>::try_from_ref(node)
                        .expect("Parent node type mismatch"))
                }
            }
        }
    }

    pub(crate) use {node, parent_definition};
}
