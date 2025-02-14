use std::{
	collections::HashMap,
	fs::{self, read_to_string},
	path::PathBuf,
};

use clap::Parser;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;

type Chain<'a> = HashMap<&'a str, ChainItem<'a>>;

/// Markov Chain generator written in Rust.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	/// The directory that contains the program inputs
	training_path: PathBuf,
}

/// Wrapper for Vec<Ustr> to make some operations easier
struct ChainItem<'a> {
	items: Vec<&'a str>,
}

impl<'a> ChainItem<'a> {
	fn new(item: &'a str) -> ChainItem<'a> {
		ChainItem { items: vec![item] }
	}

	fn add(&mut self, item: &'a str) {
		self.items.push(item);
	}

	fn merge(&mut self, other: &mut ChainItem<'a>) {
		self.items.append(&mut other.items)
	}

	fn get_rand(&self) -> &str {
		self.items
			// get a random item from the Vec
			.choose(&mut rand::thread_rng())
			.unwrap()
	}
}

fn main() {
	let args = Args::parse();

	// Gets the paths of evey file and directory in the training_path.
	let tpaths = fs::read_dir(&args.training_path)
		.unwrap_or_else(|_| panic!("Can't read files from: {:?}", args.training_path));

	// Only the files remain
	let files = tpaths
		.filter_map(|f| f.ok())
		.filter(|f| match f.file_type() {
			Err(_) => false,
			Ok(f) => f.is_file(),
		});

	// Reads every file into a string
	let contents = files.filter_map(|f| read_to_string(f.path()).ok()).map(|s| &*s.leak());

	let markov_chain = contents
		// Generates seperate chains for every string
		.map(gen_chain)
		// Then merges them
		.reduce(merge_chain)
		.expect("No chain to generate");

	// Generation
	// ~~ indicate flag
	let mut prev = "~~START";
	let mut res = String::new();
	for _ in 0..10 {
		let next = markov_chain[&prev].get_rand();
		res.push_str(&next);
		res.push(' ');
		prev = next;
	}
	res.pop();

	println!("{}", res);
}

/// Generates Markov Chain from given string
fn gen_chain<'a>(string: &'a str) -> Chain<'a> {
	// Regex for kind of tokens we want to match.
	// Matched tokens may include letters, digits, (') and (-) symbols, and can end with (.), (!), and (?) symbols.
	static WORD_REGEX: Lazy<Regex> =
		Lazy::new(|| Regex::new(r"(\w|\d|'|-)+(\.|!|\?)*").unwrap());

	let mut chain: Chain<'a> = Default::default();

	let tokens = WORD_REGEX.find_iter(string);

	// ~~ indicate flag
	let mut prev = "~~START";
	for t in tokens {
		// find_iter() doesn't return an iterator of "String"s but "Match"es. Must be converted manually.
		let t = t.as_str();

		chain.entry(prev)
			.and_modify(|ci| ci.add(t))
			.or_insert(ChainItem::new(t));

		prev = t;
	}

	chain
}

/// Merges given Markov Chains
fn merge_chain<'a>(mut a: Chain<'a>, b: Chain<'a>) -> Chain<'a> {
	for (k, mut v) in b {
		a.entry(k).and_modify(|i| i.merge(&mut v)).or_insert(v);
	}

	a
}
