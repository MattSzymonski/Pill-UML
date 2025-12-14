//! Example: Generate sequence and class diagrams from .pilluml files
//!
//! Run with: cargo run --example generate_diagrams
//!
//! ## Styling Priority (lowest to highest):
//! 1. Default styles (built into the library)
//! 2. External CSS file (via `.with_style_file()`)
//! 3. Inline CSS in `.pilluml` file (`@start_style`/`@end_style`)

use std::fs;
use std::path::Path;

fn main() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    // Check for optional shared style file
    let style_file_path = examples_dir.join("theme.css");
    let has_theme = style_file_path.exists();
    
    if has_theme {
        println!("📄 Using theme file: {}", style_file_path.display());
    }

    // Process sequence diagram using builder pattern
    let sequence_pilluml_path = examples_dir.join("sequence_example.pilluml");
    let sequence_pilluml = fs::read_to_string(&sequence_pilluml_path)
        .expect("Failed to read sequence_example.pilluml");
    
    let sequence_svg = if has_theme {
        pill_uml::create_diagram(&sequence_pilluml)
            .with_style_file(&style_file_path)
            .render()
    } else {
        pill_uml::render_diagram(&sequence_pilluml)
    };
    
    let sequence_svg_path = examples_dir.join("sequence_example.svg");
    fs::write(&sequence_svg_path, &sequence_svg).expect("Failed to write sequence diagram");
    println!("✓ Generated: {}", sequence_svg_path.display());

    // Process class diagram using builder pattern
    let class_pilluml_path = examples_dir.join("class_example.pilluml");
    let class_pilluml = fs::read_to_string(&class_pilluml_path)
        .expect("Failed to read class_example.pilluml");
    
    let class_svg = if has_theme {
        pill_uml::create_diagram(&class_pilluml)
            .with_style_file(&style_file_path)
            .render()
    } else {
        pill_uml::render_diagram(&class_pilluml)
    };
    
    let class_svg_path = examples_dir.join("class_example.svg");
    fs::write(&class_svg_path, &class_svg).expect("Failed to write class diagram");
    println!("✓ Generated: {}", class_svg_path.display());

    println!("\nDone! Open the SVG files in a browser to view.");
}
