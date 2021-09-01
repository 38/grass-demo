grass::grass_query! {
    let a = open("data/a.bed");
    let b = open("data/b.bed");
    left_outter_intersect(a, b) | filter_map(|(a, b)| b.map(|_| a)) | show_all()
}
