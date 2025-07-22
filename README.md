# readability-rust

A Rust port of [Mozilla's Readability.js](https://github.com/mozilla/readability) library for extracting readable content from web pages.

This library provides functionality to parse HTML documents and extract the main article content, removing navigation, ads, and other clutter to present clean, readable text.

## Features

- **Content Extraction**: Identifies and extracts the main article content from web pages
- **Metadata Parsing**: Extracts titles, authors, publication dates, and other metadata
- **Content Scoring**: Uses Mozilla's proven algorithms to score and rank content elements
- **Readability Assessment**: Determines if a page is likely to contain readable content
- **CLI Tool**: Command-line interface for processing HTML files and URLs
- **Multiple Output Formats**: JSON, plain text, and cleaned HTML output
- **Unicode Support**: Handles international text, emojis, and special characters
- **Error Handling**: Graceful handling of malformed HTML and edge cases

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
readability-rust = "0.1.0"
```

## Library Usage

### Basic Article Extraction

```rust
use readability_rust::{Readability, ReadabilityOptions};

let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Sample Article</title>
        <meta name="author" content="John Doe">
    </head>
    <body>
        <article>
            <h1>Article Title</h1>
            <p>This is the main content of the article...</p>
            <p>More substantial content here...</p>
        </article>
        <aside>Sidebar content to be removed</aside>
    </body>
    </html>
"#;

let mut parser = Readability::new(html, None).unwrap();
if let Some(article) = parser.parse() {
    println!("Title: {:?}", article.title);
    println!("Author: {:?}", article.byline);
    println!("Content: {:?}", article.content);
    println!("Text Length: {:?}", article.length);
}
```

### Custom Configuration

```rust
use readability_rust::{Readability, ReadabilityOptions};

let options = ReadabilityOptions {
    debug: true,
    char_threshold: 250,
    keep_classes: true,
    ..Default::default()
};

let mut parser = Readability::new(html, Some(options)).unwrap();
let article = parser.parse();
```

### Readability Assessment

```rust
use readability_rust::is_probably_readerable;

let html = "<html><body><p>Short content</p></body></html>";

if is_probably_readerable(html, None) {
    println!("This page likely contains readable content");
} else {
    println!("This page may not have substantial content");
}
```

## CLI Usage

The library includes a command-line tool for processing HTML files:

### Installation

```bash
cargo install readability-rust
```

### Basic Usage

```bash
# Process a local HTML file
readability-rust -i article.html

# Process from stdin
cat article.html | readability-rust

# Output as JSON
readability-rust -i article.html -f json

# Output as plain text
readability-rust -i article.html -f text

# Check if content is readable
readability-rust -i article.html --check

# Debug mode with verbose output
readability-rust -i article.html --debug
```

### CLI Options

```
Usage: readability-rust [OPTIONS]

Options:
  -i, --input <FILE>              Input HTML file (use '-' for stdin)
  -o, --output <FILE>             Output file (default: stdout)
  -f, --format <FORMAT>           Output format [default: json] [possible values: json, text, html]
      --base-uri <URI>            Base URI for resolving relative URLs
      --debug                     Enable debug output
      --check                     Only check if content is readable
      --char-threshold <N>        Minimum character threshold [default: 500]
      --keep-classes              Keep CSS classes in output
      --disable-json-ld           Disable JSON-LD parsing
  -h, --help                      Print help
  -V, --version                   Print version
```

## API Reference

### Core Types

#### `Readability`
The main parser struct for extracting content from HTML documents.

#### `ReadabilityOptions`
Configuration options for customizing parsing behavior:
- `debug`: Enable debug logging
- `char_threshold`: Minimum character count for content
- `keep_classes`: Preserve CSS classes in output
- `disable_json_ld`: Skip JSON-LD metadata parsing

#### `Article`
Represents extracted article content:
- `title`: Article title
- `content`: Cleaned HTML content
- `text_content`: Plain text content
- `length`: Content length in characters
- `byline`: Author information
- `excerpt`: Article excerpt/description
- `site_name`: Site name
- `lang`: Content language
- `published_time`: Publication date

### Functions

#### `is_probably_readerable(html: &str, options: Option<ReadabilityOptions>) -> bool`
Determines if an HTML document likely contains readable content.

## Algorithm

This implementation follows Mozilla's Readability.js algorithm:

1. **Preprocessing**: Remove script tags and prepare the document
2. **Content Discovery**: Identify potential content-bearing elements
3. **Scoring**: Score elements based on various factors:
   - Element types (article, p, div, etc.)
   - Class names and IDs
   - Text length and density
   - Link density
4. **Candidate Selection**: Choose the best content candidates
5. **Content Extraction**: Extract and clean the selected content
6. **Post-processing**: Final cleanup and formatting

## Testing

The library includes comprehensive tests covering:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test categories
cargo test test_article_parsing
cargo test test_metadata_extraction
cargo test test_readability_assessment
```

## Mozilla Readability Reference

This project includes the original Mozilla Readability.js library as a submodule for reference:

```bash
# Initialize the submodule
git submodule update --init --recursive

# View the original JavaScript implementation
ls mozilla-readability/
```

The original implementation can be found at: https://github.com/mozilla/readability

## Performance

The Rust implementation provides significant performance benefits:
- **Memory Safety**: No runtime memory errors
- **Zero-cost Abstractions**: Compile-time optimizations
- **Concurrent Processing**: Safe parallel processing capabilities
- **Small Binary Size**: Minimal runtime dependencies

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
git clone https://github.com/dreampuf/readability-rs.git
cd readability-rs
git submodule update --init --recursive
cargo build
cargo test
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

The original Mozilla Readability.js library is also licensed under Apache License 2.0.

## Acknowledgments

- [Mozilla Readability.js](https://github.com/mozilla/readability) - The original JavaScript implementation
- [Arc90's Readability](https://web.archive.org/web/20130627094911/https://www.readability.com/) - The original inspiration
- The Rust community for excellent crates and tooling
