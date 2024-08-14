use derive_more::{Constructor, Display};
use std::fmt::Formatter;
use itertools::Itertools;
use nonempty_collections::NEVec;
use crate::qbe::types::ExtendedType;

pub type Align = u16;

#[derive(Debug, Eq, PartialEq, Constructor)]
pub struct Field {
    ty: SubType,
    count: Option<u16>,
}

#[derive(Debug, Eq, PartialEq, Constructor)]
pub struct Layout(Vec<Field>);

#[derive(Debug, Eq, PartialEq)]
pub enum TypeDef {
    Regular {
        name: String,
        align: Option<Align>,
        fields: Layout,
    },
    Union {
        name: String,
        align: Option<Align>,
        layouts: NEVec<Layout>,
    },
    Opaque {
        name: String,
        align: Align,
        size: u16,
    },
}

impl TypeDef {
    pub fn name(&self) -> &str {
        match self {
            TypeDef::Regular { name, .. } => name,
            TypeDef::Union { name, .. } => name,
            TypeDef::Opaque { name, .. } => name
        }
    }
}

#[derive(Display, Debug, Eq, PartialEq)]
pub enum SubType {
    Extended(ExtendedType),
    User(String),
}

impl Display for TypeDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDef::Regular {
                name,
                align,
                fields,
            } => {
                writeln!(f, "type :{name} = ")?;
                if let Some(align) = align {
                    writeln!(f, "\talign {align}")?;
                }
                writeln!(f, "\t{{")?;
                writeln!(f, "\t{fields}")?;
                writeln!(f, "\t}}")?;
            }
            TypeDef::Union {
                name,
                align,
                layouts,
            } => {
                writeln!(f, "type :{name} =")?;
                if let Some(align) = align {
                    writeln!(f, "\talign {align}")?;
                }
                writeln!(f, "\t{{")?;
                for layout in layouts {
                    writeln!(f, "\t\t{{ {layout} }}")?;
                }
                writeln!(f, "\t}}")?;
            }
            TypeDef::Opaque { name, align, size } => {
                writeln!(f, "type :{name} = align {align} {{ {size} }}")?;
            }
        }
        Ok(())
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(count) = self.count {
            write!(f, "{} {count}", self.ty)
        } else {
            write!(f, "{}", self.ty)
        }
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().join(", "))
    }
}
