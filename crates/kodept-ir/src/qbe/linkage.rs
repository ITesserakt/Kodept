use crate::qbe::typedefs::Array;
use derive_more::Display;
use itertools::Itertools;

use super::typedefs::Name;

#[derive(Debug, Eq, PartialEq)]
pub struct Linkage {
    sections: Array<LinkageSection, 1>,
    export: bool,
    thread_local: bool,
}

#[derive(Display, Debug, Eq, PartialEq)]
enum LinkageSection {
    #[display("section \"{name}\"")]
    Section { name: Name },
    #[display("section \"{name}\" \"{flags}\"")]
    SectionWithFlags { name: Name, flags: String },
}

impl Linkage {
    pub fn private() -> Self {
        Self {
            export: false,
            thread_local: false,
            sections: Array::new(),
        }
    }

    pub fn public() -> Self {
        Self {
            export: true,
            thread_local: false,
            sections: Array::new(),
        }
    }

    pub fn thread_local(mut self) -> Self {
        self.thread_local = true;
        self
    }

    pub fn with_section(mut self, name: impl Into<Name>) -> Self {
        self.sections
            .push(LinkageSection::Section { name: name.into() });
        self
    }

    pub fn with_section_and_flags(mut self, name: impl Into<Name>, flags: String) -> Self {
        self.sections.push(LinkageSection::SectionWithFlags {
            name: name.into(),
            flags,
        });
        self
    }
}

impl Display for Linkage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut has_previous = false;
        if self.export {
            write!(f, "export")?;
            has_previous = true;
        }
        if self.thread_local {        
            if has_previous {
                write!(f, " ")?;
            }
            write!(f, "thread")?;
            has_previous = true;
        }
        if has_previous && !self.sections.is_empty() {
            write!(f, " ")?;
        }
        write!(f, "{}", self.sections.iter().join(" "))?;
        Ok(())
    }
}
