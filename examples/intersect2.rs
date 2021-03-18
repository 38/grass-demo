use std::env::args;
use std::fs::File;
use std::io::{BufWriter, Result, Write};

use gql::algorithm::AssumeSorted;
use gql::algorithm::SortedIntersect;
use gql::properties::Serializable;
use gql::records::{Bed3, Bed4};

use gql::{chromset::LexicalChromRef, LexicalChromSet, LineRecordStreamExt};

fn main() -> Result<()> {
    let args: Vec<_> = args().skip(1).take(3).collect();

    let chroms = LexicalChromSet::new();

    let bed3_file = File::open(&args[0])?
        .into_record_iter::<Bed3<LexicalChromRef>, _>(&chroms)
        .assume_sorted();
    let bed4_file = File::open(&args[1])?
        .into_record_iter::<Bed4<LexicalChromRef>, _>(&chroms)
        .assume_sorted();

    let mut out_file = BufWriter::new(File::create(&args[2])?);

    for pair in bed3_file.sorted_intersect(bed4_file) {
        let result = Bed3::new(&pair);
        result.dump(&mut out_file)?;
        out_file.write_all(b"\n")?;
    }

    Ok(())
}
