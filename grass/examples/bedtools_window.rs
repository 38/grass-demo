use itertools::Itertools;
use grass::algorithm::AssumeSorted; 
grass::grass_query! {
    let a = open("data/a.bed");
    let b = open("data/b.bed");
    let a_with_window = a | map({ 
        _0.begin -= 1000;
        _0.end += 1000;
    }) | assume_sorted();
    intersect(a_with_window, b) | project(0) | dedup() | show_all();
}
