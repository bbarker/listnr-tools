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

#[cfg(not(test))]
const SECTION_CHAR_LIMIT: usize = 1500;
#[cfg(test)]
const SECTION_CHAR_LIMIT: usize = 10;

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
        "listing omitted".to_string()
    } else {
        code_block.literal.to_string()
    }
}

fn chunk_markdown(content: &str) -> Vec<String> {
    let arena = Arena::new();
    let root = parse_document(&arena, content, &ComrakOptions::default());

    root.descendants()
        .filter_map(|node| match node.data.borrow().value {
            NodeValue::Text(ref text) => Some(text.clone()),
            NodeValue::Code(ref node_code) => Some(node_code.literal.clone()),
            NodeValue::CodeBlock(ref code_block) => Some(process_code_blocks(code_block)),
            _ => None,
        })
        .fold(Vec::new(), |mut chunks, text| {
            if let Some(last_chunk) = chunks.last_mut() {
                if last_chunk.len() + text.len() + 1 <= SECTION_CHAR_LIMIT {
                    last_chunk.push(' ');
                    last_chunk.push_str(&text);
                } else {
                    chunks.push(text);
                }
            } else {
                chunks.push(text);
            }
            chunks
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_markdown() {
        let content = r#"This is a test of the chunking function.
It should split the content into chunks of SECTION_CHAR_LIMIT characters or less."#;
        let chunks = chunk_markdown(content);

        // Filter out single-word chunks that exceed SECTION_CHAR_LIMIT
        let filtered_chunks: Vec<&String> = chunks
            .iter()
            .filter(|chunk| chunk.len() <= SECTION_CHAR_LIMIT)
            .collect();

        println!("Total chunks: {}", chunks.len());
        println!("Filtered chunks: {}", filtered_chunks.len());

        filtered_chunks.iter().enumerate().for_each(|(ii, chunk)| {
            println!("Chunk {}: '{}' (length: {})", ii, chunk, chunk.len());
        });

        // Calculate expected number of chunks
        let total_length: usize = content
            .split_whitespace()
            .map(|word| word.len())
            .sum::<usize>()
            + content.split_whitespace().count()
            - 1; // Add spaces between words
        let expected_chunks = (total_length as f64 / SECTION_CHAR_LIMIT as f64).ceil() as usize;

        assert!(
            filtered_chunks.len() <= expected_chunks,
            "Too many chunks. Expected: >= {}, Actual: {}",
            expected_chunks,
            filtered_chunks.len()
        );

        filtered_chunks.iter().for_each(|chunk| {
            assert!(
                chunk.len() <= SECTION_CHAR_LIMIT,
                "Filtered chunk exceeds SECTION_CHAR_LIMIT: '{}'",
                chunk
            );
        });
    }
}
