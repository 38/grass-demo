# grass-demo
Genomic Records AbStractionS

## GRASS Query DSL
### Introduction

The GRASS Query DSL allows you use the GRASS library without having a good knowledge about
Rust and GRASS library. The query language is build on top of Rust's procedual macro, and 
it allows user to write simple query language which turns to be Rust code when it gets compiled.

### Basic Syntax

To use the GRASS Query DSL, you can simply use the marco `grass::grass_query`. 
For example, you can write a rust source code:

```rust
grass::grass_query! {
	// your query DSL code here
}
```

- Let binding

You can bind any expression to variables, for example, you can use `open` to load any file on disk,
`intersect` to get a iterator of intersections, it also can be bind to a variable.

For instance:

```rust
grass::grass_query! {
	let first_file = open("a.bed");
	let second_file = open("b.bed");
	let intersected = intersect(first_file, second_file);
}
```

- Calling Method

In the GRASS DSL, you can call any assocated method / trait method defined by GRASS. 
For example, you can cast a BAM file to a BED file by calling `as_bed3` trait method.

```rust
grass::grass_query!{
	let input_file = open("path/to/file.bam");
	input_file | as_bed3() | show_all();
}
```

- Open a genomic record file

```rust
// grass-query-example.rs
grass::grass_query!{
	let input_file = open("path/to/file.bam");
}
```

note that the query DSL will automatically detect the file format and generate properate file format handling code.

- Intersect multiple files

For example, intersect two input bed file and save the result as a bed3 file.

```rust
grass::grass_query! {
	let first_file = open("a.bed");
	let second_file = open("b.bed");
	intersect(first_file, second_file) | as_bed3() | save("intersect-result.bed");
}
```

- Filtering

You can use `where` to filter the records. 
The filtering condition you can use `_1`,.... for the first, second, third intervals and `_0` for the overlapped intervals.

For example, filter out all the intervals that is shorter than 20 bases and save the result to file.

```rust
grass::grass_query! {
	let first_file = open("a.bed");
	first_file | where(_0.length() > 20) | save("filtering-result.bed");
}
```

- Mixing GRASS DSL and Rust code

You can use `grass::grass_query_block` macro for this purpose. 

```rust
use grass::grass_query_block;
fn main() {
	grass_query_block! {
		let a = open("a.bed");
		let b = open("b.bed");
		let result = intersect(a, b);
	}

	use grass::properties::*;

	for item in result {
		println!("{} {} {}", item.chrom(), item.begin(), item.end());
	}
}
```
