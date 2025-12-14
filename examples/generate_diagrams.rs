//! Example: Generate sequence and class diagrams from .pilluml files
//!
//! Run with: cargo run --example generate_diagrams
//!
//! Diagrams can include custom CSS using @start_style/@end_style blocks.
//! See the example .pilluml files for usage.

use std::fs;
use std::path::Path;

fn main() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    // Process sequence diagram
    let sequence_pilluml_path = examples_dir.join("sequence_example.pilluml");
    let sequence_pilluml = fs::read_to_string(&sequence_pilluml_path)
        .expect("Failed to read sequence_example.pilluml");
    let sequence_svg = pill_uml::render_diagram(&sequence_pilluml);
    let sequence_svg_path = examples_dir.join("sequence_example.svg");
    fs::write(&sequence_svg_path, &sequence_svg).expect("Failed to write sequence diagram");
    println!("✓ Generated: {}", sequence_svg_path.display());

    // Process class diagram
    let class_pilluml_path = examples_dir.join("class_example.pilluml");
    let class_pilluml = fs::read_to_string(&class_pilluml_path)
        .expect("Failed to read class_example.pilluml");
    let class_svg = pill_uml::render_diagram(&class_pilluml);
    let class_svg_path = examples_dir.join("class_example.svg");
    fs::write(&class_svg_path, &class_svg).expect("Failed to write class diagram");
    println!("✓ Generated: {}", class_svg_path.display());

    println!("\nDone! Open the SVG files in a browser to view.");
}
