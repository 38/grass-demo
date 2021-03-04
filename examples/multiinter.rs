use std::env::args;
use std::fs::File;
use std::io::Result;

use gql::algorithm::{AssumeSorted, Components};
use gql::ChromName;
use gql::records::Bed3;

use gql::{LineRecordStreamExt, LexicalChromSet, chromset::LexicalChromRef};

use itertools::{kmerge, Itertools};

fn main() -> Result<()> {
    let args: Vec<_> = args().skip(1).collect();
    
    let chroms = LexicalChromSet::new();

    let inputs: Vec<_> = args[0..args.len()]
        .iter()
        .enumerate()
        .map(|(file_id, path)| {
            File::open(path)
                .unwrap()
                .into_record_iter::<Bed3<LexicalChromRef>, _>(&chroms)
                .assume_sorted()
                .components()
                .map(move |x| (x, file_id))
        })
        .collect();
    let mut current_depth = vec![0; inputs.len()];
    let mut active_count = 0;
    let mut last_pos = None;

    for ((chr, pos), group) in kmerge(inputs)
        .group_by(|(comp, _)| {
            // TODO: this is ineffcient, we need to handle chrom name better to avoid copying strings
            // everywhere.
            let (chr, pos) = comp.position();
            (chr.clone(), pos)
        })
        .into_iter()
    {
        if let Some((left_chr, left_pos)) = last_pos {
            //TODO: Rust's println macro is very slow
            if left_chr == chr && active_count > 0 {
                println!("{}\t{}\t{}\t{:?}", chr.to_string(), left_pos, pos, current_depth);
            }
        }

        for (comp, file_idx) in group {
            match (current_depth[file_idx], comp.depth) {
                (0, new) if new > 0 => active_count += 1,
                (old, 0) if old > 0 => active_count -= 1,
                _ => (),
            }
            current_depth[file_idx] = comp.depth;
        }

        last_pos = Some((chr, pos));
    }
    Ok(())
}
