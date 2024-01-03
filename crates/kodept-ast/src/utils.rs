#[macro_export]
macro_rules! make_ast_node_adaptor {
    ($name:ident, lifetimes: [$($life:lifetime$(,)*)*], $wrapper:ident, configs: [$($cfg:meta$(,)*)*]) => {
        $(#[$cfg])*
        pub enum $name<$($life, )*> {
            File($wrapper<$($life, )* FileDeclaration>),
            Module($wrapper<$($life, )* ModuleDeclaration>),
            Struct($wrapper<$($life, )* StructDeclaration>),
            Enum($wrapper<$($life, )* EnumDeclaration>),
            Type($wrapper<$($life, )* Type>),
            Parameter($wrapper<$($life, )* Parameter>),
            TypedParameter($wrapper<$($life, )* TypedParameter>),
            UntypedParameter($wrapper<$($life, )* UntypedParameter>),
            TopLevel($wrapper<$($life, )* TopLevel>),
            TypeName($wrapper<$($life, )* TypeName>),
            Variable($wrapper<$($life, )* Variable>),
            InitializedVariable($wrapper<$($life, )* InitializedVariable>),
            BodiedFunction($wrapper<$($life, )* BodiedFunctionDeclaration>),
            Body($wrapper<$($life, )* Body>),
            ExpressionBlock($wrapper<$($life, )* ExpressionBlock>),
            Operation($wrapper<$($life, )* Operation>),
            Application($wrapper<$($life, )* Application>),
            Lambda($wrapper<$($life, )* Lambda>),
            Expression($wrapper<$($life, )* Expression>),
            Term($wrapper<$($life, )* Term>),
            Reference($wrapper<$($life, )* Reference>),
            Access($wrapper<$($life, )* Access>),
            Number($wrapper<$($life, )* NumberLiteral>),
            Char($wrapper<$($life, )* CharLiteral>),
            String($wrapper<$($life, )* StringLiteral>),
            Tuple($wrapper<$($life, )* TupleLiteral>),
            Literal($wrapper<$($life, )* Literal>),
            CodeFlow($wrapper<$($life, )* CodeFlow>),
            If($wrapper<$($life, )* IfExpression>),
            Elif($wrapper<$($life, )* ElifExpression>),
            Else($wrapper<$($life, )* ElseExpression>),
            Binary($wrapper<$($life, )* Binary>),
            Unary($wrapper<$($life, )* Unary>),
            AbstractFunction($wrapper<$($life, )* AbstractFunctionDeclaration>),
            ResolvedReference($wrapper<$($life, )* ResolvedReference>),
            ResolvedTypeReference($wrapper<$($life, )* ResolvedTypeReference>),
            ProdType($wrapper<$($life, )* ProdType>),
            SumType($wrapper<$($life, )* SumType>),
            Identifier($wrapper<$($life, )* Identifier>),
        }

        impl<$($life, )*> kodept_core::Named for $name<$($life, )*> {}
    };
}

#[macro_export]
macro_rules! property {
    (in mut $trait_name:ty => $self_name:ty, $prop:ident: $prop_ty:ty) => {
        paste::paste! {
        impl $trait_name for $self_name {
            fn [<get_ $prop>](&self) -> $prop_ty {
                self.$prop
            }

            fn [<set_ $prop>](&mut self, value: $prop_ty) {
                self.$prop = value;
            }
        }
        }
    };
    (in $trait_name:ty => $self_name:ty, $prop:ident: $prop_ty:ty) => {
        paste::paste! {
        impl $trait_name for $self_name {
            fn [<get_ $prop>](&self) -> $prop_ty {
                self.$prop
            }
        }
        }
    };
}
