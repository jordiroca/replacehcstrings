// replacehcstrings - segunda parte del extractor de cadenas
// Replaces hardcoded string in an HTML file with Handlebars syntax to an hbs file

// Convierte un archivo HTML en un archivo Handlebars (.hbs) extrayendo
// las cadenas "hard-coded" y sacándolas a un archivo .{lang}.json

// Recibe las cadenas "hard-coded" en un archivo JSON desde el lint.

// Autor: Jordi Roca
// Fecha: 08/04/2024


use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use unidecode::unidecode;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    error: Error,
}

#[derive(Serialize, Deserialize, Debug)]
struct Error {
    line: usize,
    evidence: String,
    character: usize,
    scope: String,
}

fn slugify(text: &str) -> String {
    let mut slug = unidecode(text)
        .to_lowercase()
        .replace(" ", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    if slug.len() > 64 {
        slug.truncate(64);
    }

    slug
}

// Le llamo evidence a la cadena hardcoded (heredado de lint)

fn clean_evidence(evidence: &str) -> String {
    let re = Regex::new(r"^/(.*?)/[m]*$").unwrap();
    if let Some(caps) = re.captures(evidence) {
        if let Some(matched) = caps.get(1) {
            return matched.as_str().to_string();
        }
    }
    evidence.to_string()  // Return the original string if no match is found
}

fn process_files(
    html_file_path: &Path,
    json_file_path: &Path,
    lang: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let replacements: Vec<Item> = serde_json::from_reader(fs::File::open(json_file_path)?)?;

    let mut html_content = fs::read_to_string(html_file_path)?;

    for item in &replacements {
        let line_number = item.error.line - 1;
        // let evidence = item.error.evidence.trim_matches('/').trim_matches('(').trim_matches(')').replace("\\", "");
        let evidence = clean_evidence(&item.error.evidence);
        println!("evidence: {}", evidence);

        // let evidence_pattern = Regex::new(&format!(r"(?P<left>.*?)(?P<evidence>{})(?P<right>.*)", regex::escape(&evidence)))?;
        let evidence_pattern = Regex::new(&format!(r"(?P<left>.*?)(?P<evidence>{})(?P<right>.*)", &evidence))?;
        let _start_char = item.error.character - 1;
        let _scope_line = &item.error.scope;

        if let Some(line) = html_content.lines().nth(line_number) {
            let new_line = if let Some(captures) = evidence_pattern.captures(&line) {
                // println!("captures: {:?}", captures);
                let left = captures.name("left").map(|m| m.as_str()).unwrap_or("");
                let right = captures.name("right").map(|m| m.as_str()).unwrap_or("");
                let replacement_text = format!("{}{{{{t '{}'}}}}{}", left, slugify(&evidence), right);
                replacement_text
            } else {
                println!("No match found for evidence: {}", evidence);
                line.to_string()
            };
            html_content = html_content.replace(&line, &new_line);
        }
    }
    let html_basename = html_file_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let json_file_name = format!("{}.{}.json", html_basename, lang);
    let mut replacement_mappings: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for item in replacements {
        let evidence = item.error.evidence.trim_matches('/').replace("\\", "");
        let replacement_text = evidence.trim_matches('(').trim_matches(')');
        replacement_mappings.insert(
            slugify(&replacement_text),
            String::from(evidence.trim_matches('(').trim_matches(')')),
        );
    }
    fs::write(
        json_file_name,
        serde_json::to_string_pretty(&replacement_mappings)?,
    )?;

    Ok(html_content)
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: replacehcstrings <html_file_path> <json_file_path> [language]");
        return;
    }

    let html_file_path = Path::new(&args[1]);
    let json_file_path = Path::new(&args[2]);
    let lang = args.get(3).unwrap_or(&"es".to_string()).to_string();

    match process_files(html_file_path, json_file_path, &lang) {
        Ok(modified_content) => {
            let output_file_path = html_file_path.with_extension("hbs");
            fs::write(&output_file_path, modified_content.clone()).unwrap();
            println!("File processed and saved successfully.");
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
