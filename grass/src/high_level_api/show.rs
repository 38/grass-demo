use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    fs::File,
    io::{BufWriter, Write},
    iter::Take,
    marker::PhantomData,
    ops::{Add, Range},
    path::Path,
};

use num::iter::RangeFrom;

use crate::{
    properties::{Intersection, Serializable, WithRegionCore},
    ChromName,
};

pub struct Show<Iter: Iterator> {
    iter: RefCell<Iter>,
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

pub trait PrintOpt {
    fn append<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        intersection: &D,
        delim: &str,
        mut target: W,
    ) -> std::io::Result<()> {
        target.write_all(delim.as_bytes())?;
        self.print(intersection, delim, target)
    }
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        intersection: &D,
        delim: &str,
        target: W,
    ) -> std::io::Result<()>;

    fn delim<'a>(self, delim: &'a str) -> DeliminatorOverride<'a, Self>
    where
        Self: Sized,
    {
        DeliminatorOverride(self, delim)
    }
}

pub struct DeliminatorOverride<'a, T>(T, &'a str);

impl<'a, T: PrintOpt> PrintOpt for DeliminatorOverride<'a, T> {
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        intersection: &D,
        _: &str,
        target: W,
    ) -> std::io::Result<()> {
        self.0.print(intersection, self.1, target)
    }
}

fn write_interval_impl<C: ChromName, D: WithRegionCore<C>, W: Write>(
    interval: &D,
    delim: &str,
    mut target: W,
) -> std::io::Result<()> {
    write!(
        target,
        "{chrom}{delim}{begin}{delim}{end}",
        chrom = interval.chrom().to_string(),
        begin = interval.begin(),
        end = interval.end(),
        delim = delim,
    )
}

pub struct Overlap;

impl PrintOpt for Overlap {
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        interval: &D,
        delim: &str,
        target: W,
    ) -> std::io::Result<()> {
        write_interval_impl(interval, delim, target)
    }
}
impl<C: PrintOpt> Add<C> for Overlap {
    type Output = PrintOptPair<Self, C>;
    fn add(self, rhs: C) -> Self::Output {
        PrintOptPair(self, rhs)
    }
}

pub struct Original<T>(pub T);

impl PrintOpt for Original<usize> {
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        intersection: &D,
        delim: &str,
        target: W,
    ) -> std::io::Result<()> {
        if self.0 < intersection.size() {
            let interval = intersection.original(self.0).to_bed3();
            write_interval_impl(&interval, delim, target)
        } else {
            Ok(())
        }
    }
}
impl<C: PrintOpt, T> Add<C> for Original<T>
where
    Self: PrintOpt,
{
    type Output = PrintOptPair<Self, C>;
    fn add(self, rhs: C) -> Self::Output {
        PrintOptPair(self, rhs)
    }
}

pub trait OverlapRef: Iterator<Item = usize> + Clone {}

impl OverlapRef for Range<usize> {}
impl OverlapRef for RangeFrom<usize> {}

impl<T: OverlapRef> PrintOpt for Original<T> {
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        intersection: &D,
        delim: &str,
        mut target: W,
    ) -> std::io::Result<()> {
        let mut first = true;
        for idx in self.0.clone() {
            if idx < intersection.size() {
                let interval = intersection.original(idx).to_bed3();
                if !first {
                    target.write_all(delim.as_bytes())?;
                }
                write_interval_impl(&interval, delim, &mut target)?;
            } else {
                break;
            }
            first = false;
        }
        Ok(())
    }
}

pub struct S<'a>(pub &'a str);

impl<'a> PrintOpt for S<'a> {
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        _: &D,
        _: &str,
        mut target: W,
    ) -> std::io::Result<()> {
        write!(target, "{}", self.0)
    }
}
impl<'a, C: PrintOpt> Add<C> for S<'a> {
    type Output = PrintOptPair<Self, C>;
    fn add(self, rhs: C) -> Self::Output {
        PrintOptPair(self, rhs)
    }
}

pub struct Fraction(pub usize);

impl PrintOpt for Fraction {
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        intersection: &D,
        _delim: &str,
        mut target: W,
    ) -> std::io::Result<()> {
        if self.0 < intersection.size() {
            let total_size = intersection.original(self.0).length();
            let overlap_size = intersection.length();
            write!(target, "{:.5}", overlap_size as f64 / total_size as f64)
        } else {
            Ok(())
        }
    }
}
impl<C: PrintOpt> Add<C> for Fraction {
    type Output = PrintOptPair<Self, C>;
    fn add(self, rhs: C) -> Self::Output {
        PrintOptPair(self, rhs)
    }
}

pub struct PrintOptPair<A: PrintOpt, B: PrintOpt>(A, B);

impl<A: PrintOpt, B: PrintOpt> PrintOpt for PrintOptPair<A, B> {
    fn print<W: Write, C: ChromName, D: Intersection<C>>(
        &self,
        data: &D,
        delim: &str,
        mut target: W,
    ) -> std::io::Result<()> {
        self.0.print(data, delim, &mut target)?;
        self.1.append(data, delim, target)
    }
}

impl<A: PrintOpt, B: PrintOpt, C: PrintOpt> Add<C> for PrintOptPair<A, B> {
    type Output = PrintOptPair<Self, C>;
    fn add(self, rhs: C) -> Self::Output {
        PrintOptPair(self, rhs)
    }
}

pub struct IntersectionPrinting<C: ChromName, I: Iterator, P: PrintOpt> {
    iter: RefCell<I>,
    config: P,
    _phantom_data: PhantomData<C>,
}

impl<C, I, P> Debug for IntersectionPrinting<C, I, P>
where
    C: ChromName,
    I: Iterator,
    I::Item: Intersection<C>,
    P: PrintOpt,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut iter_ref = self.iter.borrow_mut();
        let mut count = 0;
        let mut buf = Vec::new();
        while let Some(item) = iter_ref.next() {
            buf.clear();
            self.config.print(&item, "\t", &mut buf).unwrap();
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

pub trait IntersectionCatExt<C: ChromName>
where
    Self: Iterator + Sized,
    Self::Item: Intersection<C>,
{
    fn cat<P: PrintOpt>(self, print_config: P) -> IntersectionPrinting<C, Self, P> {
        IntersectionPrinting {
            iter: RefCell::new(self),
            config: print_config,
            _phantom_data: PhantomData,
        }
    }

    fn head<P: PrintOpt>(
        self,
        n: usize,
        print_config: P,
    ) -> IntersectionPrinting<C, std::iter::Take<Self>, P> {
        IntersectionPrinting {
            iter: RefCell::new(self.take(n)),
            config: print_config,
            _phantom_data: PhantomData,
        }
    }

    fn tail<P: PrintOpt>(
        self,
        n: usize,
        print_config: P,
    ) -> IntersectionPrinting<C, std::collections::vec_deque::IntoIter<Self::Item>, P> {
        let mut last = std::collections::VecDeque::new();

        for item in self {
            last.push_back(item);
            if last.len() > n {
                last.pop_front();
            }
        }

        IntersectionPrinting {
            iter: RefCell::new(last.into_iter()),
            config: print_config,
            _phantom_data: PhantomData,
        }
    }
}

impl<C, T> IntersectionCatExt<C> for T
where
    C: ChromName,
    T: Iterator + Sized,
    Self::Item: Intersection<C>,
{
}
