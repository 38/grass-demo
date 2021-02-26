use super::Bed3;
use crate::{ChromName, ChromSetHandle, properties::{/*Parsable, Serializable, */WithName, WithRegion}};

//use std::io::{Result, Write};
use std::rc::Rc;

#[derive(Clone)]
pub struct Bed4<T: ChromName> {
    core: Bed3<T>,
    name: Rc<String>,
}

impl<T: ChromName> WithRegion<T> for Bed4<T> {
    fn begin(&self) -> u32 {
        self.core.begin()
    }

    fn end(&self) -> u32 {
        self.core.end()
    }

    fn chrom(&self) -> T {
        self.core.chrom.clone()
    }
}

impl<T: ChromName> WithName for Bed4<T> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

/*impl Parsable for Bed4<String> {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        let core = Bed3::parse(tokens)?;
        Some(Self {
            core,
            name: Rc::new(tokens.next()?.to_string()),
        })
    }
}*/

/*impl<T: ChromName> Serializable for Bed4<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.core.dump(&mut fp)?;
        fp.write(b"\t")?;
        fp.write_all(self.name().as_bytes())
    }
}*/

impl<T: ChromName> Bed4<T> {
    pub fn with_chrom_list<H:ChromSetHandle>(self, chrom_list: &mut H) -> Bed4<H::RefType> {
        Bed4 {
            core: self.core.with_chrom_list(chrom_list),
            name: self.name,
        }
    }
}
