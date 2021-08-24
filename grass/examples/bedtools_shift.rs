grass::grass_query! {
    let a = open("data/a.bed");
    a | map({
        _0.begin -= 10;
        _0.end -= 10;
    }) | show_all();
}
