use std::{io::Write, thread_local};

use crate::{
    algorithm::Point,
    chromset::LexicalChromRef,
    properties::{Serializable, WithRegion},
    records::Bed3,
};

pub use super::*;

mod open_helper;
pub use open_helper::*;

mod show;
pub use show::*;

mod merge;
pub use merge::*;

mod depth;
pub use depth::*;

// TODO: because we use unsafe cell, so that we actually need a mutex inside the chromset for
// thread safety. But we currently doesn't have any. as long as we are currently single
// threaded, it's Ok for now but we definitely should change it later
thread_local! {
    static CHROM_NAMES : LexicalChromSet = {
        LexicalChromSet::new()
    };
}

pub fn get_global_chrom_list() -> &'static std::thread::LocalKey<LexicalChromSet> {
    &CHROM_NAMES
}

pub trait AsBed3: IntoIterator + Sized + 'static
where
    Self::Item: WithRegion<LexicalChromRef>,
{
    fn as_bed3(self) -> Box<dyn Iterator<Item = Bed3<LexicalChromRef>>> {
        Box::new(self.into_iter().map(|item| Bed3::new(item)))
    }
}

impl<T: IntoIterator + 'static> AsBed3 for T where T::Item: WithRegion<LexicalChromRef> {}

impl<T: WithRegion<LexicalChromRef> + Serializable> Serializable for Point<LexicalChromRef, T> {
    fn dump<W: Write>(&self, mut fp: W) -> std::io::Result<()> {
        let pos = self.position();
        pos.0.write(&mut fp)?;
        let is_open = if self.is_open { "open" } else { "close" };
        fp.write_all(format!("\t{}\t{}\t{}\t|\t", pos.1, self.depth, is_open).as_bytes())?;
        self.value.dump(&mut fp)
    }
}
