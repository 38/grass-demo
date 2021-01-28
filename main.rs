//mod algorithms;
mod algorithm;
mod properties;
mod records;

use std::io::{BufRead, BufReader, Result, BufWriter, Write};
use std::fs::File;
use std::env::args;

use records::{Bed3, Bed4};
use properties::{Parsable, Serializable};
use algorithm::Intersect;

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

    let bed3_file = parse_file::<Bed3<String>>(&args[0])?.map(|x| x.unwrap());

    #[cfg(not(feature = "d4-hts"))]
    let bed4_file = parse_file::<Bed4<String>>(&args[1])?.map(|x| x.unwrap());

    let mut out_file = BufWriter::new(File::create(&args[2])?);

    for pair in bed3_file.intersect(bed4_file) {
        let result = Bed3::new(&pair);
        result.dump(&mut out_file)?;
        out_file.write_all(b"\n")?;
    }

    Ok(())
}
