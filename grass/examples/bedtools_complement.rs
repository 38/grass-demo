use grass::algorithm::AssumeSorted;
grass::grass_query! {
    let a = open("data/a.bed");
    let g = open("data/test.genome");
    // The invert operator assume the genome size is INF, thus we need to intersect with
    // the genome size file, so that we can limit the result not larger than the genome size
    intersect(a | invert() | assume_sorted(), g) | as_bed3() | show_all();
}
