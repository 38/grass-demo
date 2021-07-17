use crate::{
    chromset::LexicalChromRef,
    properties::{Parsable, Serializable, WithName, WithRegionCore, WithScore, WithStrand},
};
use crate::{ChromName, ChromSetHandle, WithChromSet};
use std::io::{Result, Write};

#[derive(Clone, Copy)]
pub struct Bed3<T: ChromName = LexicalChromRef> {
    pub begin: u32,
    pub end: u32,
    pub chrom: T,
}

impl<T: ChromName> Bed3<T> {
    pub fn with_chrom_list<H: ChromSetHandle>(self, chrom_list: &mut H) -> Bed3<H::RefType> {
        let chrom = chrom_list.query_or_insert(self.chrom.to_string().as_ref());
        Bed3 {
            begin: self.begin,
            end: self.end,
            chrom,
        }
    }
}

impl<T: ChromName, H: ChromSetHandle> WithChromSet<H> for Bed3<T> {
    type Result = Bed3<H::RefType>;
    fn with_chrom_set(self, handle: &mut H) -> Self::Result {
        self.with_chrom_list(handle)
    }
}

impl<'a> Parsable<'a> for Bed3<&'a str> {
    fn parse(s: &'a str) -> Option<(Self, usize)> {
        let mut bytes = s.as_bytes();

        if bytes.last() == Some(&b'\n') {
            bytes = &bytes[..bytes.len() - 1];
        }

        let mut token_pos_iter = memchr::Memchr::new(b'\t', bytes);
        let end_1 = token_pos_iter.next()?;
        let end_2 = token_pos_iter.next()?;
        let end_3 = token_pos_iter.next().unwrap_or(bytes.len());
        let chrom = &s[..end_1];

        Some((
            Self {
                chrom,
                begin: s[end_1 + 1..end_2].parse().ok()?,
                end: s[end_2 + 1..end_3].parse().ok()?,
            },
            end_3,
        ))
    }
}

impl<C: ChromName + Clone> Bed3<C> {
    pub fn new<T: WithRegionCore<C>>(region: T) -> Self {
        Self {
            begin: region.begin(),
            end: region.end(),
            chrom: region.chrom().clone(),
        }
    }
}

impl<T: ChromName> Serializable for Bed3<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        self.chrom().write(&mut fp)?;
        fp.write(b"\t")?;
        crate::ioutils::write_number(&mut fp, self.begin() as i32)?;
        fp.write(b"\t")?;
        crate::ioutils::write_number(&mut fp, self.end() as i32).map(|_| ())
    }
}

impl<T: ChromName> WithRegionCore<T> for Bed3<T> {
    fn begin(&self) -> u32 {
        self.begin
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn chrom(&self) -> &T {
        &self.chrom
    }
}

impl<T: ChromName> WithName for Bed3<T> {
    fn name(&self) -> &str {
        "."
    }
}
impl<T: ChromName> WithScore<i32> for Bed3<T> {}
impl<T: ChromName> WithStrand for Bed3<T> {}
