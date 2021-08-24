grass::grass_query! {
    let a = open("data/a.bed");
    let b = open("data/b.bed");
    intersect(a, b) | cat(
        Overlap + S("|")
        + Original(0) + Fraction(0) +S("|")
        + Original(1) + Fraction(1)
    )
}
