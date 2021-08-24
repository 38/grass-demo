grass::grass_query! {
    let a = open("data/a.bed");
    a | merge_overlaps() | show_all();
}
