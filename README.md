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

- 


