use super::Bed3;
use crate::{
    chromset::LexicalChromRef,
    properties::{Parsable, Serializable, WithName, WithRegion, WithScore, WithStrand},
    ChromName, ChromSetHandle, WithChromSet,
};

use std::io::{Result, Write};
use std::rc::Rc;

#[derive(Clone)]
pub struct Bed4<T: ChromName = LexicalChromRef> {
    pub core: Bed3<T>,
    pub name: Rc<String>,
}

impl<T: ChromName, H: ChromSetHandle> WithChromSet<H> for Bed4<T> {
    type Result = Bed4<H::RefType>;
    fn with_chrom_set(self, handle: &mut H) -> Self::Result {
        let core = self.core.with_chrom_list(handle);
        let name = self.name;
        Bed4 { core, name }
    }
}

impl<'a> Parsable<'a> for Bed4<&'a str> {
    fn parse(s: &'a str) -> Option<(Self, usize)> {
        let (core, rem) = Bed3::parse(s)?;
        let rem = s[rem + 1..]
            .split(|c| c == '\t' || c == '\n')
            .next()
            .unwrap_or(".")
            .to_string();
        Some((
            Self {
                core,
                name: Rc::new(rem),
            },
            s.len(),
        ))
    }
}

impl<T: ChromName> WithRegion<T> for Bed4<T> {
    fn begin(&self) -> u32 {
        self.core.begin()
    }

    fn end(&self) -> u32 {
        self.core.end()
    }

    fn chrom(&self) -> &T {
        self.core.chrom()
    }
}

impl<T: ChromName> WithName for Bed4<T> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<T: ChromName> Serializable for Bed4<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.core.dump(&mut fp)?;
        fp.write(b"\t")?;
        fp.write_all(self.name().as_bytes())
    }
}

impl<T: ChromName> Bed4<T> {
    pub fn with_chrom_list<H: ChromSetHandle>(self, chrom_list: &mut H) -> Bed4<H::RefType> {
        Bed4 {
            core: self.core.with_chrom_list(chrom_list),
            name: self.name,
        }
    }
}

impl<T: ChromName> WithScore<i32> for Bed4<T> {}
impl<T: ChromName> WithStrand for Bed4<T> {}
