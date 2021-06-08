grass::grass_query! {
    let a = open("data/a.bed");
    let b = open("data/b.bed");
    intersect(a, b) | as_bed3() | save("/tmp/result.bed");
}
