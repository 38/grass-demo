use std::{io::Write, marker::PhantomData, thread_local};

use crate::{
    algorithm::Point,
    chromset::LexicalChromRef,
    properties::{Serializable, WithRegion},
    records::Bed3,
};

use self::properties::Intersection;

pub use super::*;

mod open_helper;
pub use open_helper::*;

mod show;
pub use show::*;

mod merge;
pub use merge::*;

mod depth;
pub use depth::*;

mod subtract;
pub use subtract::*;

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

pub struct Projection<T, C>(T, usize, PhantomData<C>);
impl<T, C> Iterator for Projection<T, C>
where
    T: Iterator,
    T::Item: Intersection<C>,
    C: ChromName,
{
    type Item = Bed3<C>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.0.next() {
            if self.1 < next.size() {
                let prj = next.original(self.1);
                return Some(prj.to_bed3());
            }
        }
        None
    }
}

pub trait Project: IntoIterator + Sized {
    fn project<C: ChromName>(self, n: usize) -> Projection<Self::IntoIter, C>
    where
        Self::Item: Intersection<C>,
    {
        Projection(self.into_iter(), n, Default::default())
    }
}

impl<T: IntoIterator> Project for T {}
