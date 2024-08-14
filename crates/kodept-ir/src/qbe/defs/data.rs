use std::fmt::{Display, Formatter};
use std::num::NonZeroU64;
use derive_more::Constructor;
use itertools::Itertools;
use nonempty_collections::NEVec;
use crate::qbe::constants::Constant;
use crate::qbe::defs::aggregate::Align;
use crate::qbe::linkage::Linkage;
use crate::qbe::types::ExtendedType;

#[derive(Debug, PartialEq)]
pub enum DataItem {
    Symbol {
        name: String,
        offset: Option<NonZeroU64>
    },
    Text(String),
    Constant(Constant)
}

#[derive(Debug, PartialEq)]
pub enum DataChunk {
    Zeros(NonZeroU64),
    Filled {
        ty: ExtendedType,
        items: NEVec<DataItem>
    }
}

#[derive(Debug, PartialEq, Constructor)]
pub struct DataDef {
    linkage: Linkage,
    name: String,
    align: Option<Align>,
    chunks: Vec<DataChunk>
}

impl Display for DataItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataItem::Symbol { name, offset } => {
                if let Some(offset) = offset {
                    write!(f, "${name} + {offset}")
                } else {
                    write!(f, "${name}")
                }
            }
            DataItem::Text(s) => write!(f, "\"{s}\""),
            DataItem::Constant(c) => write!(f, "{c}")
        }
    }
}

impl Display for DataChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataChunk::Zeros(count) => write!(f, "z {count}"),
            DataChunk::Filled { ty, items } => {
                write!(f, "{ty} {}", items.into_iter().join(" "))
            }
        }
    }
}

impl Display for DataDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.linkage != Linkage::private() {
            write!(f, "{} ", self.linkage)?;
        }
        write!(f, "data ${} =", self.name)?;
        if let Some(align) = self.align {
            write!(f, " align {align}")?;
        }
        write!(f, " {{ ")?;
        write!(f, "{}", self.chunks.iter().join(", "))?;
        write!(f, " }}")?;
        Ok(())
    }
}
