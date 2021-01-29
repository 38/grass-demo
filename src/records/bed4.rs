use super::Bed3;
use crate::properties::{WithRegion, WithName, Parsable, Serializable};

use std::io::{Write, Result};

#[derive(Clone)]
pub struct Bed4<T: AsRef<str>> {
    core: Bed3<T>,
    name: T,
}

impl<T: AsRef<str>> WithRegion for Bed4<T> {
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

impl<T: AsRef<str>> WithName for Bed4<T> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Parsable for Bed4<String> {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        let core = Bed3::parse(tokens)?;
        Some(Self {
            core,
            name: tokens.next()?.to_string(),
        })
    }
}

impl<T: AsRef<str>> Serializable for Bed4<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.core.dump(&mut fp)?;
        fp.write(b"\t")?;
        fp.write_all(self.name().as_bytes())
    }
}

impl<'a> Bed4<&'a str> {
    #[allow(dead_code)]
    pub fn with_name<T: WithRegion + 'a>(region: &'a T, name: &'a str) -> Self {
        Self {
            core: Bed3::new(region),
            name,
        }
    }
}
