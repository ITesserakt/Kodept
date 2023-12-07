#[cfg(feature = "generic-lifetime")]
#[macro_export]
macro_rules! declare_visitor {
    ($visitor:ty, $family:ty, $meta:ty, $output:ty) => {
        impl $crate::Visitor<$family> for $visitor {
            type Data<'d> = $meta;
            type Output = $output;
        }
    };
}

#[cfg(not(feature = "generic-lifetime"))]
#[macro_export]
macro_rules! declare_visitor {
    ($visitor:ty, $family:ty, $meta:ty, $output:ty) => {
        impl $crate::Visitor<$family> for $visitor {
            type Data = $meta;
            type Output = $output;
        }
    };
}

#[macro_export]
macro_rules! node_group {
    {family: $family:ty, nodes: [$($node:ty$(,)*)*]} => {
        impl<V: $crate::Visitor<Self>> $crate::NodeFamily<V> for $family {}

        $(impl<V: $crate::Visitor<$family> + $crate::Visit<$family, $node>> $crate::Node<$family, V> for $node {
        })+
    };
    {family: $family:ty, nodes: [$($node:ty$(,)*)+]} => {
        $crate::node_group!(family: $family, nodes: [$($node)+], meta: ())
    };
}

#[macro_export]
macro_rules! impl_visitor {
    {$self:ty, family: $family:ty, output: $output:ty, meta: $meta:ty, [$($node:ty => $visits:expr$(,)*)*]} => {
        $crate::declare_visitor!($self, $family, $meta, $output);

        $(impl $crate::Visit<$family, $node> for $self {
            #[allow(unused_variables)]
            fn visit(&mut self, node: &mut $node, metadata: $crate::Data<$family, $self>) -> Self::Output {
                let closure: fn(&mut Self, &mut $node, $crate::Data<$family, Self>) -> Self::Output = $visits;
                #[allow(clippy::redundant_closure_call)]
                closure(self, node, metadata)
            }
        })*
    };
}
