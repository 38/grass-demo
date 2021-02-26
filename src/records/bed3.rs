//use crate::chrom::{Chrom, ChromList, ChromListRef};
//
use crate::{ChromName,ChromSetHandle};
use crate::properties::{/*Parsable, Serializable, */ WithRegion};
//use std::io::{Result, Write};

#[derive(Clone)]
pub struct Bed3<T: ChromName> {
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

impl<'a> Bed3<&'a str> {
    pub fn new<T: WithRegion<&'a str>>(region: &'a T) -> Self {
        Self {
            begin: region.begin(),
            end: region.end(),
            chrom: region.chrom(),
        }
    }
}

/*impl Parsable for Bed3<String> {
    fn parse<'a, T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        Some(Self {
            chrom: tokens.next()?.to_owned(),
            begin: tokens.next()?.parse().ok()?,
            end: tokens.next()?.parse().ok()?,
        })
    }
}

impl <'a> Parsable<'a> for Bed3<&'a str> {
    fn parse<T: Iterator<Item = &'a str>>(tokens: &mut T) -> Option<Self> {
        Some(Self {
            chrom: tokens.next()?,
            begin: tokens.next()?.parse().ok()?,
            end: tokens.next()?.parse().ok()?,
        })
    }
}

fn write_number<W: Write>(mut fp: W, mut n: u32) -> Result<()> {
    if n == 0 {
        fp.write(b"0").map(|_| ())
    } else {
        let mut buf = [0; 10];
        let mut offset = 0;
        let mut begin = 0;
        while n > 0 {
            buf[offset] = b'0' + (n % 10) as u8;
            n /= 10;
            offset += 1;
        }
        let mut end = offset - 1;
        while begin < end {
            buf.swap(begin, end);
            begin += 1;
            end -= 1;
        }

        fp.write(&buf[..offset]).map(|_| ())
    }
}*/

/*impl<T: Chrom> Serializable for Bed3<T> {
    fn dump<W: Write>(&self, mut fp: W) -> Result<()> {
        fp.write_all(self.chrom().as_bytes())?;
        fp.write(b"\t")?;
        write_number(&mut fp, self.begin())?;
        fp.write(b"\t")?;
        write_number(&mut fp, self.end()).map(|_| ())
    }
}*/

impl<T: ChromName> WithRegion<T> for Bed3<T> {
    fn begin(&self) -> u32 {
        self.begin
    }

    fn end(&self) -> u32 {
        self.end
    }

    fn chrom(&self) -> T {
        self.chrom.clone()
    }
}
