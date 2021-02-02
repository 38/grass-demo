use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result, Write};

use gql::algorithm::AssumeSorted;
use gql::algorithm::Components;
use gql::properties::{Parsable, Serializable};
use gql::records::Bed3;

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

    let mut last_begin = None;

    for comp in bed3_file.components() {
        if let Some((ref chr, pos)) = last_begin {
            if comp.depth == 0 {
               let result = Bed3 {
                   chrom: chr,
                   begin: pos,
                   end: comp.position().1
               };
               result.dump(&mut out_file)?;
               out_file.write_all(b"\n")?;
               last_begin = None;
            }
        } else {
            if comp.depth > 0 {
                let (chr, pos) = comp.position();
                last_begin = Some((chr.to_owned(), pos));
            }
        }
    }
    Ok(())
}
