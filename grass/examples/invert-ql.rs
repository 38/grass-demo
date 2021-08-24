grass::grass_query! {
    let a = open("data/a.bed");
    a | invert() | show_all();
}
