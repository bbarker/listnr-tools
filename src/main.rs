use comrak::{nodes::NodeValue, parse_document, Arena, ComrakOptions};
use regex::Regex;
use std::env;
use std::fs;

const SECTION_CHAR_LIMIT: usize = 1500;

fn main() {
    // Read the input file from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_markdown_file>", args[0]);
        std::process::exit(1);
    }
    let input_file = &args[1];

    // Read the content of the Markdown file
    let content = fs::read_to_string(input_file).expect("Failed to read the input file");

    // Replace code segments with the placeholder
    let code_block_regex = Regex::new(r"```.*?```").unwrap();
    let content =
        code_block_regex.replace_all(&content, "listing omitted; please see the original source");

    // Parse the Markdown content
    let arena = Arena::new();
    let root = parse_document(&arena, &content, &ComrakOptions::default());

    // Collect paragraphs and sections
    let mut chunks = vec![];
    let mut current_chunk = String::new();

    root.descendants().for_each(|node| {
        if let NodeValue::Text(ref text) = node.data.borrow().value {
            if current_chunk.len() + text.len() > SECTION_CHAR_LIMIT {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }
            current_chunk.push_str(text);
        }
    });
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks.iter().for_each(|chunk| {
        println!("{}\n---\n", chunk);
    });
}
