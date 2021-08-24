use itertools::Itertools;

grass::grass_query! {
    let a = open("data/a.bed");
    let b = open("data/b.bed");
    intersect(a, b) | where(_1.length() as f64 / _0.length() as f64 > 0.10) | show_all();
}
