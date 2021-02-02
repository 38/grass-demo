use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result, Write};

use gql::algorithm::AssumeSorted;
use gql::algorithm::SortedIntersect;
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
    let args: Vec<_> = args().skip(1).take(4).collect();

    let file1 = parse_file::<Bed3<String>>(&args[0])?
        .map(|x| x.unwrap())
        .assume_sorted();

    let file2 = parse_file::<Bed3<String>>(&args[1])?
        .map(|x| x.unwrap())
        .assume_sorted();

    let file3 = parse_file::<Bed3<String>>(&args[2])?
        .map(|x| x.unwrap())
        .assume_sorted();

    let mut out_file = BufWriter::new(File::create(&args[3])?);

    for triple in file1.sorted_intersect(file2).sorted_intersect(file3) {
        let result = Bed3::new(&triple);
        result.dump(&mut out_file)?;
        out_file.write_all(b"\n")?;
    }

    Ok(())
}
