//! Example: Generate sequence and class diagrams from .puml files
//!
//! Run with: cargo run --example generate_diagrams

use std::fs;
use std::path::Path;

fn main() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    // Load optional base style file
    let style_path = examples_dir.join("style.iuml");
    let base_style = fs::read_to_string(&style_path).ok();
    
    if base_style.is_some() {
        println!("📄 Using base style from: {}", style_path.display());
    }

    // Process sequence diagram
    let sequence_puml_path = examples_dir.join("sequence_example.puml");
    let sequence_puml = fs::read_to_string(&sequence_puml_path)
        .expect("Failed to read sequence_example.puml");
    let sequence_svg = pill_uml::render_with_base_style(&sequence_puml, base_style.as_deref());
    let sequence_svg_path = examples_dir.join("sequence_example.svg");
    fs::write(&sequence_svg_path, &sequence_svg).expect("Failed to write sequence diagram");
    println!("✓ Generated: {}", sequence_svg_path.display());

    // Process class diagram
    let class_puml_path = examples_dir.join("class_example.puml");
    let class_puml = fs::read_to_string(&class_puml_path)
        .expect("Failed to read class_example.puml");
    let class_svg = pill_uml::render_with_base_style(&class_puml, base_style.as_deref());
    let class_svg_path = examples_dir.join("class_example.svg");
    fs::write(&class_svg_path, &class_svg).expect("Failed to write class diagram");
    println!("✓ Generated: {}", class_svg_path.display());

    println!("\nDone! Open the SVG files in a browser to view.");
}
