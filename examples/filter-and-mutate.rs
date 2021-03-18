use std::env::args;
use std::fs::File;
use std::io::{BufWriter, Result, Write};

use gql::properties::{Serializable, WithRegion};
use gql::records::Bed3;
use gql::{chromset::LexicalChromRef, LexicalChromSet, LineRecordStreamExt};

fn main() -> Result<()> {
    let args: Vec<_> = args().skip(1).take(3).collect();

    let chroms = LexicalChromSet::new();

    let bed3_file = File::open(&args[0])?.into_record_iter::<Bed3<LexicalChromRef>, _>(&chroms);
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
