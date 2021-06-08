use std::env::args;
use std::fs::File;
use std::io::{BufWriter, Result, Write};

use grass::algorithm::AssumeSorted;
use grass::algorithm::Components;
use grass::properties::Serializable;
use grass::records::Bed3;
use grass::{chromset::LexicalChromRef, LexicalChromSet, LineRecordStreamExt};

use itertools::Itertools;

fn main() -> Result<()> {
    let args: Vec<_> = args().skip(1).take(3).collect();

    let chroms = LexicalChromSet::new();

    let bed3_file = File::open(&args[0])?
        .into_record_iter::<Bed3<LexicalChromRef>, _>(&chroms)
        .assume_sorted();

    let mut out_file = BufWriter::new(File::create(&args[1])?);

    let mut id = 0;

    for (first, last) in bed3_file
        .components()
        .group_by(move |x| {
            if x.depth == 0 {
                id += 1;
                id - 1
            } else {
                id
            }
        })
        .into_iter()
        .map(|(_, mut overlaps)| (overlaps.next().unwrap(), overlaps.last().unwrap()))
    {
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
