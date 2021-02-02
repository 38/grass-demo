use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result, Write};

use gql::algorithm::AssumeSorted;
use gql::algorithm::Components;
use gql::properties::{Parsable, Serializable};
use gql::records::Bed3;

use itertools::Itertools;

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

    let bed3_file = parse_file::<Bed3<String>>(&args[0])?
        .map(|x| x.unwrap())
        .assume_sorted();
    
    let mut out_file = BufWriter::new(File::create(&args[1])?);

    let mut id = 0;

    for (first, last) in bed3_file.components()
        .group_by(move |x| if x.depth == 0 { id += 1; id - 1 } else { id })
        .into_iter()
        .map(|(_, mut overlaps)| (overlaps.next().unwrap(), overlaps.last().unwrap())) {
        let result = Bed3 {
            chrom: first.position().0,
            begin: first.position().1,
            end: last.position().1,
        };
        result.dump(&mut out_file)?;
        out_file.write_all(b"\n")?;
    }


    Ok(())
}
