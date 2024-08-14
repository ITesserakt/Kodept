pub mod constants;
pub mod control;
pub mod defs;
pub mod linkage;
pub mod module;
pub mod types;

pub mod typedefs {
    use smallvec::SmallVec;

    pub type Array<T, const N: usize = 2> = SmallVec<[T; N]>;
    pub type Name = String;
}

#[cfg(test)]
mod tests {
    use crate::qbe::constants::{Constant, Value};
    use crate::qbe::control::block::{Block, Jump};
    use crate::qbe::control::instruction::{add, call};
    use crate::qbe::defs::data::{DataChunk, DataDef, DataItem};
    use crate::qbe::defs::funcs::{Function, Parameter};
    use crate::qbe::linkage::Linkage;
    use crate::qbe::module::Module;
    use crate::qbe::types::{Byte, Word};
    use nonempty_collections::nev;

    use super::control::instruction::Argument;
    use super::types::Long;

    fn build_hello_world() -> Module<'static> {
        let fn_add = Function::new(
            Linkage::private(),
            "add".to_string(),
            nev![Block::new("start")
                .with_instr(add::smtm(
                    Value::local("c"),
                    Word,
                    [Value::local("a"), Value::local("b")]
                ))
                .with_jump(Jump::Return(Value::local("c")))],
        )
        .with_return_type(Word)
        .with_param(Parameter::regular(Word, "a"))
        .with_param(Parameter::regular(Word, "b"));

        let fn_main = Function::new(
            Linkage::public(),
            "main".to_string(),
            nev![Block::new("start")
                .with_instr(call::assignment(
                    Value::local("r"),
                    Word,
                    Value::global("add"),
                    [Argument::regular(Word, 1), Argument::regular(Word, 1)]
                ))
                .with_instr(call::stmt(
                    Value::global("printf"),
                    [
                        Argument::regular(Long, Value::global("fmt")),
                        Argument::Variadic,
                        Argument::regular(Word, Value::local("r"))
                    ]
                ))
                .with_jump(Jump::ret(0))],
        )
        .with_return_type(Word);

        let data_fmt = DataDef::new(
            Linkage::private(),
            "fmt".to_string(),
            None,
            vec![
                DataChunk::Filled {
                    ty: Byte.into(),
                    items: nev![DataItem::Text("One and one make %d!\\n".to_string())],
                },
                DataChunk::Filled {
                    ty: Byte.into(),
                    items: nev![DataItem::Constant(Constant::Integer(0))],
                },
            ],
        );

        Module::new()
            .with_fn(fn_add)
            .with_fn(fn_main)
            .with_data(data_fmt)
    }

    #[test]
    fn test_display_impl() {
        let module = build_hello_world();
        let text = module.to_string();

        similar_asserts::assert_eq!(
            text,
            r###"function w $add(w %a, w %b) {
@start
	%c =w add %a, %b
	ret %c
}
export function w $main() {
@start
	%r =w call $add(w 1, w 1)
	call $printf(l $fmt, ..., w %r)
	ret 0
}
data $fmt = { b "One and one make %d!\n", b 0 }
"###
        );
    }
}
