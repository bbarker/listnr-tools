use clap::Parser;
use comrak::nodes::NodeCodeBlock;
use comrak::{nodes::NodeValue, parse_document, Arena, ComrakOptions};
use csv::Reader;
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
    substitutions.iter().for_each(|(from, to)| {
        result = result.replace(from, to);
    });
    result
}

fn process_code_blocks(code_block: &NodeCodeBlock) -> String {
    if code_block.literal.len() > 80 {
        "listing omitted; please see the original source".to_string()
    } else {
        code_block.literal.to_string()
    }
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
                Some(process_code_blocks(code_block))
            } else {
                None
            }
        })
        .fold(Vec::<String>::new(), |mut chunks, text| {
            let mut current_chunk = chunks.pop().unwrap_or_default();
            if current_chunk.len() + text.len() > SECTION_CHAR_LIMIT {
                chunks.push(current_chunk);
                chunks.push(text);
            } else {
                current_chunk.push_str(&text);
                chunks.push(current_chunk);
            }
            chunks
        })
}
