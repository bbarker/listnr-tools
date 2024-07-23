use clap::Parser;
use comrak::{nodes::NodeValue, parse_document, Arena, ComrakOptions};
use csv::Reader;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input markdown file
    #[arg(short, long)]
    input: PathBuf,

    /// Optional CSV file for word substitution
    #[arg(short, long)]
    substitutions: Option<PathBuf>,
}

const SECTION_CHAR_LIMIT: usize = 1500;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Read the input markdown file
    let mut content = fs::read_to_string(&args.input)?;

    // Process substitutions if CSV file is provided
    if let Some(csv_path) = args.substitutions {
        let substitutions = read_substitutions(&csv_path)?;
        content = apply_substitutions(&content, &substitutions);
    }

    let chunks = chunk_markdown(&content);

    // Print the chunks
    chunks.iter().for_each(|chunk| {
        println!("\n--- --- --- {} --- --- ---\n{}", chunk.len(), chunk);
    });

    Ok(())
}

fn read_substitutions(path: &PathBuf) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(path)?;
    let mut substitutions = HashMap::new();

    rdr.records().try_for_each(|result| {
        let record = result?;
        if record.len() == 2 {
            substitutions.insert(record[0].to_string(), record[1].to_string());
        }
        Ok::<(), csv::Error>(())
    })?;

    Ok(substitutions)
}

fn apply_substitutions(content: &str, substitutions: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    for (from, to) in substitutions {
        result = result.replace(from, to);
    }
    result
}

fn process_code_blocks(content: &str) -> String {
    let code_block_regex = Regex::new(r"```[\s\S]*?```").unwrap();
    code_block_regex
        .replace_all(content, |caps: &regex::Captures| {
            let code_block = caps.get(0).unwrap().as_str();
            if code_block.len() > 80 {
                "listing omitted; please see the original source".to_string()
            } else {
                code_block.to_string()
            }
        })
        .to_string()
}

fn chunk_markdown(content: &str) -> Vec<String> {
    let arena = Arena::new();
    let root = parse_document(&arena, content, &ComrakOptions::default());

    root.descendants()
        .filter_map(|node| {
            if let NodeValue::Text(ref text) = node.data.borrow().value {
                Some(text.clone())
            } else if let NodeValue::Code(ref node_code) = node.data.borrow().value {
                Some(node_code.literal.clone())
            } else if let NodeValue::CodeBlock(ref code_block) = node.data.borrow().value {
                Some(process_code_blocks(&code_block.literal.clone()))
            } else {
                None
            }
        })
        .fold(
            (Vec::new(), String::new()),
            |(mut chunks, mut current_chunk), text| {
                if current_chunk.len() + text.len() > SECTION_CHAR_LIMIT {
                    chunks.push(current_chunk);
                    current_chunk = String::new();
                }
                current_chunk.push_str(&text);
                (chunks, current_chunk)
            },
        )
        .0
}

/*

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
*/
