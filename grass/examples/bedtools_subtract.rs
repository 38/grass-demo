grass::grass_query! {
    let a = open("data/a.bed");
    let b = open("data/b.bed");
    a | subtract(b) | as_bed3() | show_all();
}
