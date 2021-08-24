use itertools::Itertools;

grass::grass_query! {
    let a = open("data/a.bed");
    let b = open("data/b.bed");
    intersect(a, b) | project(0) | dedup() | show_all();
}
