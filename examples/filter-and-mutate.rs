use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result, Write};

use gql::properties::{Parsable, Serializable, WithRegion};
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

    let bed3_file = parse_file::<Bed3<String>>(&args[0])?.map(|x| x.unwrap());
    let mut out_file = BufWriter::new(File::create(&args[1])?);

    for result in bed3_file.filter(|x| x.length() > 50).map(|mut x| {
        x.begin -= 1000;
        x
    }) {
        result.dump(&mut out_file).ok();
        out_file.write_all(b"\n").ok();
    }

    Ok(())
}
