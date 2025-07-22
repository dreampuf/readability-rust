//! Command-line interface for the Readability library

use clap::{Arg, Command};
use readability_rust::{Readability, ReadabilityOptions, is_probably_readerable};
use serde_json;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

#[derive(Debug)]
struct CliOptions {
    input: Option<String>,
    output: Option<String>,
    format: OutputFormat,
    base_uri: Option<String>,
    debug: bool,
    check_only: bool,
    char_threshold: usize,
    keep_classes: bool,
    disable_json_ld: bool,
}

#[derive(Debug, Clone)]
enum OutputFormat {
    Json,
    Text,
    Html,
}

impl From<&str> for OutputFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "text" => OutputFormat::Text,
            "html" => OutputFormat::Html,
            _ => OutputFormat::Json, // Default
        }
    }
}

fn main() {
    let matches = Command::new("readability")
        .version("0.1.0")
        .author("Mozilla Readability Rust Port")
        .about("Extract article content from web pages, removing clutter like ads and navigation")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input HTML file (use '-' for stdin)")
                .required(false)
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file (default: stdout)")
                .required(false)
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format: json, text, html")
                .default_value("json")
                .required(false)
        )
        .arg(
            Arg::new("base-uri")
                .short('b')
                .long("base-uri")
                .value_name("URI")
                .help("Base URI for resolving relative URLs")
                .required(false)
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Enable debug output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .help("Only check if document is readable (exit code 0=readable, 1=not readable)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("min-content-length")
                .long("min-content-length")
                .value_name("LENGTH")
                .help("Minimum content length for readability check")
                .default_value("140")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("char-threshold")
                .long("char-threshold")
                .value_name("CHARS")
                .help("Minimum character threshold for article content")
                .default_value("500")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("keep-classes")
                .long("keep-classes")
                .help("Keep CSS classes in output")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("disable-json-ld")
                .long("disable-json-ld")
                .help("Disable JSON-LD parsing for metadata")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let cli_options = CliOptions {
        input: matches.get_one::<String>("input").cloned(),
        output: matches.get_one::<String>("output").cloned(),
        format: OutputFormat::from(matches.get_one::<String>("format").unwrap().as_str()),
        base_uri: matches.get_one::<String>("base-uri").cloned(),
        debug: matches.get_flag("debug"),
        check_only: matches.get_flag("check"),
        char_threshold: *matches.get_one::<usize>("char-threshold").unwrap(),
        keep_classes: matches.get_flag("keep-classes"),
        disable_json_ld: matches.get_flag("disable-json-ld"),
    };

    if let Err(e) = run(cli_options) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(options: CliOptions) -> Result<(), Box<dyn std::error::Error>> {
    // Read input HTML
    let html = read_input(&options.input)?;
    
    if options.debug {
        eprintln!("Read {} characters of HTML", html.len());
    }

    // If check-only mode, just test readability
    if options.check_only {
        let readable = is_probably_readerable(&html, None);
        if options.debug {
            eprintln!("Document is {}readable", if readable { "" } else { "not " });
        }
        process::exit(if readable { 0 } else { 1 });
    }

    // Create readability options
    let readability_options = ReadabilityOptions {
        debug: options.debug,
        char_threshold: options.char_threshold,
        keep_classes: options.keep_classes,
        disable_json_ld: options.disable_json_ld,
        ..Default::default()
    };

    // Create readability parser
    let mut readability = if let Some(base_uri) = &options.base_uri {
        Readability::new_with_base_uri(&html, base_uri, Some(readability_options))?
    } else {
        Readability::new(&html, Some(readability_options))?
    };

    // Parse the document
    let article = readability.parse();
    
    match article {
        Some(article) => {
            let output = format_output(&article, &options.format)?;
            write_output(&output, &options.output)?;
            
            if options.debug {
                eprintln!("Successfully extracted article:");
                eprintln!("  Title: {}", article.title.as_deref().unwrap_or("None"));
                eprintln!("  Length: {} characters", article.length.unwrap_or(0));
            }
        }
        None => {
            eprintln!("Failed to extract article content from the document");
            process::exit(1);
        }
    }

    Ok(())
}

fn read_input(input: &Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    match input {
        Some(path) if path == "-" => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
        Some(path) => {
            // Read from file
            if !Path::new(path).exists() {
                return Err(format!("Input file '{}' does not exist", path).into());
            }
            fs::read_to_string(path).map_err(|e| e.into())
        }
        None => {
            // Read from stdin if no input specified
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn write_output(content: &str, output: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    match output {
        Some(path) => {
            fs::write(path, content)?;
        }
        None => {
            print!("{}", content);
        }
    }
    Ok(())
}

fn format_output(
    article: &readability_rust::Article,
    format: &OutputFormat,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(article)?;
            Ok(json)
        }
        OutputFormat::Text => {
            let mut output = String::new();
            
            if let Some(title) = &article.title {
                output.push_str(&format!("Title: {}\n\n", title));
            }
            
            if let Some(byline) = &article.byline {
                output.push_str(&format!("By: {}\n\n", byline));
            }
            
            if let Some(text_content) = &article.text_content {
                output.push_str(text_content);
            }
            
            Ok(output)
        }
        OutputFormat::Html => {
            let mut output = String::new();
            output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
            
            if let Some(title) = &article.title {
                output.push_str(&format!("    <title>{}</title>\n", html_escape(title)));
            }
            
            output.push_str("    <meta charset=\"utf-8\">\n");
            output.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
            output.push_str("</head>\n<body>\n");
            
            if let Some(title) = &article.title {
                output.push_str(&format!("    <h1>{}</h1>\n", html_escape(title)));
            }
            
            if let Some(byline) = &article.byline {
                output.push_str(&format!("    <p class=\"byline\">By {}</p>\n", html_escape(byline)));
            }
            
            if let Some(content) = &article.content {
                output.push_str("    <div class=\"content\">\n");
                output.push_str(content);
                output.push_str("\n    </div>\n");
            }
            
            output.push_str("</body>\n</html>\n");
            Ok(output)
        }
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert!(matches!(OutputFormat::from("json"), OutputFormat::Json));
        assert!(matches!(OutputFormat::from("text"), OutputFormat::Text));
        assert!(matches!(OutputFormat::from("html"), OutputFormat::Html));
        assert!(matches!(OutputFormat::from("invalid"), OutputFormat::Json)); // Default
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("Hello & <World>"), "Hello &amp; &lt;World&gt;");
        assert_eq!(html_escape("\"Test\""), "&quot;Test&quot;");
    }

    #[test]
    fn test_format_output_json() {
        let article = readability_rust::Article {
            title: Some("Test Title".to_string()),
            content: Some("<p>Test content</p>".to_string()),
            text_content: Some("Test content".to_string()),
            length: Some(12),
            excerpt: None,
            byline: Some("Test Author".to_string()),
            dir: None,
            site_name: None,
            lang: None,
            published_time: None,
            readerable: Some(true),
        };

        let result = format_output(&article, &OutputFormat::Json).unwrap();
        assert!(result.contains("Test Title"));
        assert!(result.contains("Test content"));
        assert!(result.contains("Test Author"));
    }

    #[test]
    fn test_format_output_text() {
        let article = readability_rust::Article {
            title: Some("Test Title".to_string()),
            content: Some("<p>Test content</p>".to_string()),
            text_content: Some("Test content".to_string()),
            length: Some(12),
            excerpt: None,
            byline: Some("Test Author".to_string()),
            dir: None,
            site_name: None,
            lang: None,
            published_time: None,
            readerable: Some(true),
        };

        let result = format_output(&article, &OutputFormat::Text).unwrap();
        assert!(result.contains("Title: Test Title"));
        assert!(result.contains("By: Test Author"));
        assert!(result.contains("Test content"));
    }

    #[test]
    fn test_format_output_html() {
        let article = readability_rust::Article {
            title: Some("Test Title".to_string()),
            content: Some("<p>Test content</p>".to_string()),
            text_content: Some("Test content".to_string()),
            length: Some(12),
            excerpt: None,
            byline: Some("Test Author".to_string()),
            dir: None,
            site_name: None,
            lang: None,
            published_time: None,
            readerable: Some(true),
        };

        let result = format_output(&article, &OutputFormat::Html).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>Test Title</title>"));
        assert!(result.contains("<h1>Test Title</h1>"));
        assert!(result.contains("By Test Author"));
        assert!(result.contains("<p>Test content</p>"));
    }
}