mod algorithms;
mod properties;
mod records;

use algorithms::{intersect_sorted_file, Intersect};
use properties::{Parsable, Serializable, WithRegion};
use records::{Bed3, Bed4};
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result, Write};

fn parse_file<T: Parsable>(path: &str) -> Result<impl Iterator<Item = Option<T>>> {
    let file = BufReader::new(File::open(path)?);
    Ok(file.lines().map(|line| {
        if let Ok(line) = line {
            return T::parse(&mut line.split('\t'));
        }
        None
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = args().skip(1).take(3).collect();

    let bed3_file = parse_file::<Bed3>(&args[0])?.map(|x| x.unwrap());
    #[cfg(not(feature = "d4-hts"))]
    let bed4_file = parse_file::<Bed4>(&args[1])?.map(|x| x.unwrap());

    #[cfg(feature = "d4-hts")]
    let bam_file = d4_hts::BamFile::open(&args[1]).unwrap();

    #[cfg(feature = "d4-hts")]
    let bed4_file = records::BAMRecord::iter_of(&bam_file);

    let mut out_file = BufWriter::new(File::create(&args[2])?);

    intersect_sorted_file(bed3_file, bed4_file, |a, b| {
        let intersection = a.intersect(b);
        if !intersection.empty() {
            intersection.dump(&mut out_file).ok();
            out_file.write(b"\n").ok();
        }
        true
    });
    Ok(())
}
