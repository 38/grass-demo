use super::Bed3;
use crate::chrom::{Chrom, ChromList, ChromListRef};
use crate::properties::{Parsable, Serializable, WithName, WithRegion};

use std::io::{Result, Write};
use std::rc::Rc;

#[derive(Clone)]
pub struct Bed4<T: Chrom> {
    core: Bed3<T>,
    name: Rc<String>,
}

impl<T: Chrom> WithRegion for Bed4<T> {
    fn begin(&self) -> u32 {
        self.core.begin()
    }

    fn end(&self) -> u32 {
        self.core.end()
    }

    fn chrom(&self) -> &str {
        self.core.chrom()
    }
}

impl<T: Chrom> WithName for Bed4<T> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Parsable for Bed4<String> {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        let core = Bed3::parse(tokens)?;
        Some(Self {
            core,
            name: Rc::new(tokens.next()?.to_string()),
        })
    }
}

impl<T: Chrom> Serializable for Bed4<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.core.dump(&mut fp)?;
        fp.write(b"\t")?;
        fp.write_all(self.name().as_bytes())
    }
}

impl<T: Chrom + Into<String>> Bed4<T> {
    pub fn with_chrom_list(self, chrom_list: &ChromList) -> Bed4<ChromListRef> {
        Bed4 {
            core: self.core.with_chrom_list(chrom_list),
            name: self.name,
        }
    }
}
