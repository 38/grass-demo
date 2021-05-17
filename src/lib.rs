pub mod algorithm;
//mod chrom;
//pub use chrom::{Chrom, ChromList, ChromListRef};

pub mod properties;
pub mod records;

pub mod chromset;
pub use chromset::{ChromName, ChromSet, ChromSetHandle, LexicalChromSet, WithChromSet};

mod file;
pub use file::{LineRecordStream, LineRecordStreamExt};

pub(crate) mod ioutils;

pub mod high_level_api {
    pub use crate::{
        algorithm::{
            AssumeSorted, AssumingSortedIter, Components, ComponentsIter, Point, Sorted,
            SortedIntersect, TaggedComponentExt,
        },
        chromset::LexicalChromRef,
        properties::{Serializable, WithRegion},
        records::{Bed3, Bed4, Bed5},
    };
    use std::{
        cell::RefCell,
        collections::HashMap,
        fmt::{Debug, Formatter},
        fs::File,
        hash::Hash,
        io::BufWriter,
        iter::Take,
        path::Path,
        rc::Rc,
    };
    use std::{io::Write, thread_local};

    use self::algorithm::TaggedComponent;

    pub use super::*;

    // TODO: because we use unsafe cell, so that we actually need a mutex inside the chromset for
    // thread safety. But we currently doesn't have any. as long as we are currently single
    // threaded, it's Ok for now but we definitely should change it later
    thread_local! {
        static CHROM_NAMES : LexicalChromSet = {
            LexicalChromSet::new()
        };
    }

    macro_rules! define_open_helper {
        ($name:ident, $($record_type:tt)*) => {
            pub fn $name<P: AsRef<Path>>(path: P) -> AssumingSortedIter<impl Iterator<Item = $($record_type)*>> {
                CHROM_NAMES.with(|chrom_names| {
                    std::fs::File::open(path).map(|file| file.into_record_iter::<$($record_type)*, _>(chrom_names).assume_sorted())
                }).unwrap()
            }
        };
    }
    define_open_helper!(open_sorted_bed3, Bed3);
    define_open_helper!(open_sorted_bed4, Bed4);
    define_open_helper!(open_sorted_bed5, Bed5<LexicalChromRef, f64>);

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

    pub struct Show<Iter: Iterator> {
        iter: RefCell<Iter>,
    }

    impl<T: WithRegion<LexicalChromRef> + Serializable> Serializable for Point<LexicalChromRef, T> {
        fn dump<W: Write>(&self, mut fp: W) -> std::io::Result<()> {
            let pos = self.position();
            pos.0.write(&mut fp)?;
            let is_open = if self.is_open { "open" } else { "close" };
            fp.write_all(format!("\t{}\t{}\t{}\t|\t", pos.1, self.depth, is_open).as_bytes())?;
            self.value.dump(&mut fp)
        }
    }

    impl<Iter: Iterator> Debug for Show<Iter>
    where
        Iter::Item: Serializable,
    {
        fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
            let mut buf = Vec::new();
            let mut iter_ref = self.iter.borrow_mut();
            let mut count = 0;
            while let Some(item) = iter_ref.next() {
                buf.clear();
                item.dump(&mut buf).ok();
                write!(f, "{}\n", String::from_utf8_lossy(&buf))?;
                count += 1;
            }
            write!(
                f,
                "-------------------------------\n{} Rows Returned",
                count
            )?;
            Ok(())
        }
    }
    pub trait ShowExt
    where
        Self: IntoIterator + Sized,
    {
        fn show_top(self, n: usize) -> Show<Take<Self::IntoIter>> {
            Show {
                iter: RefCell::new(self.into_iter().take(n)),
            }
        }
        fn show_all(self) -> Show<Self::IntoIter> {
            Show {
                iter: RefCell::new(self.into_iter()),
            }
        }
        fn save<P: AsRef<Path>>(self, path: P) -> std::io::Result<()>
        where
            Self::Item: Serializable,
        {
            let mut out = BufWriter::new(File::create(path)?);
            for item in self {
                item.dump(&mut out)?;
                out.write_all("\n".as_bytes())?;
            }
            Ok(())
        }
    }

    impl<T: IntoIterator + Sized> ShowExt for T where T::Item: Serializable {}

    pub struct Merger<I: Iterator + Sorted>
    where
        I::Item: WithRegion<LexicalChromRef> + Clone,
    {
        peek: Option<Point<LexicalChromRef, I::Item>>,
        iter: ComponentsIter<LexicalChromRef, I>,
    }

    impl<I: Iterator + Sorted> Iterator for Merger<I>
    where
        I::Item: WithRegion<LexicalChromRef> + Clone,
    {
        type Item = Bed3<LexicalChromRef>;
        fn next(&mut self) -> Option<Self::Item> {
            let mut begin = None;
            let mut end;
            while self.peek.as_ref().map_or(false, |peek| peek.depth > 0) {
                let mut buf = self.iter.next();
                std::mem::swap(&mut buf, &mut self.peek);
                end = Some(buf.unwrap().value.clone());
                if begin.is_none() {
                    begin = end.clone();
                }
            }
            end = self.peek.take().map(|x| x.value);
            self.peek = self.iter.next();
            if let Some(ret) = begin {
                let mut ret = Bed3::new(ret);
                ret.end = end.unwrap().end();
                Some(ret)
            } else {
                None
            }
        }
    }

    pub struct TaggedMerger<I, T> {
        iter: I,
        chrom: Option<LexicalChromRef>,
        begins: HashMap<T, u32>,
    }

    impl<I, R, T> Iterator for TaggedMerger<I, T>
    where
        I: Iterator<Item = (T, Point<LexicalChromRef, R>)>,
        R: WithRegion<LexicalChromRef> + Clone,
        T: ToString + Eq + Hash,
    {
        type Item = Bed4<LexicalChromRef>;
        fn next(&mut self) -> Option<Self::Item> {
            while let Some((tag, comp)) = self.iter.next() {
                let (chr, pos) = comp.position();
                if Some(&chr) != self.chrom.as_ref() {
                    self.begins.clear();
                }
                self.chrom = Some(chr);
                if let Some(&begin) = self.begins.get(&tag) {
                    if !comp.is_open {
                        let core = Bed3 {
                            chrom: self.chrom.clone().unwrap(),
                            begin,
                            end: pos,
                        };
                        let result = Bed4 {
                            core,
                            name: Rc::new(tag.to_string()),
                        };
                        self.begins.remove(&tag);
                        return Some(result);
                    }
                } else {
                    self.begins.entry(tag).or_insert(pos);
                }
            }
            None
        }
    }

    pub trait MergeExt
    where
        Self: IntoIterator + Sized,
        <Self as IntoIterator>::IntoIter: Sorted,
        <Self as IntoIterator>::Item: WithRegion<LexicalChromRef> + Clone,
    {
        fn merge_overlaps(self) -> Merger<<Self as IntoIterator>::IntoIter> {
            let mut iter = self.into_iter().components();
            let peek = iter.next();
            Merger { iter, peek }
        }
        fn tagged_merge<T: Clone + Hash + Eq, F: FnMut(&Self::Item) -> T>(
            self,
            f: F,
        ) -> TaggedMerger<
            TaggedComponent<
                LexicalChromRef,
                ComponentsIter<LexicalChromRef, Self::IntoIter>,
                Self::Item,
                T,
                F,
            >,
            T,
        > {
            TaggedMerger {
                iter: self.into_iter().components().with_tag(f),
                begins: HashMap::new(),
                chrom: None,
            }
        }
    }
    impl<T: IntoIterator + Sized> MergeExt for T
    where
        T::IntoIter: Sorted,
        T::Item: WithRegion<LexicalChromRef> + Clone,
    {
    }

    pub struct DepthIter<I: Iterator>
    where
        I::Item: WithRegion<LexicalChromRef> + Clone,
    {
        last: Option<Point<LexicalChromRef, I::Item>>,
        iter: ComponentsIter<LexicalChromRef, I>,
    }

    impl<I: Iterator> Iterator for DepthIter<I>
    where
        I::Item: WithRegion<LexicalChromRef> + Clone,
    {
        type Item = Bed5<LexicalChromRef, usize>;
        fn next(&mut self) -> Option<Self::Item> {
            let last = self.last.take()?;
            if let Some(next) = self.iter.next() {
                let (last_chr, last_pos) = last.position();
                let (next_chr, next_pos) = next.position();
                if last_chr == next_chr {
                    let result = Bed5::new(last_chr, last_pos, next_pos, ".", last.depth);
                    self.last = Some(next);
                    return Some(result);
                } else {
                    self.last = Some(next);
                    return self.next();
                }
            }
            None
        }
    }

    pub trait DepthExt
    where
        Self: IntoIterator + Sized,
        <Self as IntoIterator>::Item: WithRegion<LexicalChromRef> + Clone,
    {
        fn coverage(self) -> DepthIter<Self::IntoIter> {
            let mut iter = self.into_iter().components();
            let last = iter.next();
            DepthIter { iter, last }
        }
    }

    impl<T> DepthExt for T
    where
        T: IntoIterator + Sized,
        T::Item: WithRegion<LexicalChromRef> + Clone,
    {
    }
}
