//! # Pill UML
//!
//! A pure Rust PlantUML diagram renderer that generates SVG output.
//! No Java or external dependencies required.
//!
//! ## Supported Diagram Types
//!
//! - **Sequence Diagrams**: participants, messages, alt/else blocks, notes, dividers
//! - **Class Diagrams**: classes, interfaces, fields, methods, relationships
//!
//! ## Quick Start
//!
//! ```rust
//! use pill_uml::render_diagram;
//!
//! let source = r#"
//! @startuml
//! participant Client
//! participant Server
//! Client -> Server: Request
//! Server --> Client: Response
//! @enduml
//! "#;
//!
//! let svg = render_diagram(source);
//! ```
//!
//! ## Custom Styling
//!
//! ```rust,ignore
//! use pill_uml::{render_diagram_styled, DiagramStyle};
//!
//! let style = DiagramStyle::default()
//!     .with_font_family("Consolas, monospace")
//!     .with_background_color("#1e1e1e")
//!     .with_font_color("#d4d4d4");
//!
//! let source = "@startuml\n@enduml";
//! let svg = render_diagram_styled(source, &style);
//! ```

mod common;
mod class_diagram;
mod sequence_diagram;

pub use common::{DiagramStyle, DiagramType};
pub use class_diagram::{ClassDiagram, ClassDef, RelationType};
pub use sequence_diagram::{SequenceDiagram, Participant, Message, ArrowStyle};

/// Detect the diagram type from PlantUML source
pub fn detect_diagram_type(source: &str) -> DiagramType {
    if class_diagram::is_class_diagram(source) {
        DiagramType::Class
    } else {
        DiagramType::Sequence
    }
}

/// Render a PlantUML diagram to SVG with default styling
///
/// Automatically detects whether it's a sequence or class diagram.
pub fn render_diagram(source: &str) -> String {
    render_diagram_styled(source, &DiagramStyle::default())
}

/// Render a PlantUML diagram to SVG with custom styling
pub fn render_diagram_styled(source: &str, style: &DiagramStyle) -> String {
    match detect_diagram_type(source) {
        DiagramType::Sequence => sequence_diagram::render(source, style),
        DiagramType::Class => class_diagram::render(source, style),
    }
}

/// Render with PlantUML skinparam style string (for compatibility)
pub fn render_with_skinparams(source: &str, skinparams: &str) -> String {
    let style = DiagramStyle::from_skinparams(skinparams);
    render_diagram_styled(source, &style)
}

/// Render with optional base style file
/// 
/// Base style provides defaults, but skinparams in the source file override them.
pub fn render_with_base_style(source: &str, base_style: Option<&str>) -> String {
    let mut style = DiagramStyle::default();
    
    // Apply base style first (if provided)
    if let Some(base) = base_style {
        style = DiagramStyle::from_skinparams(base);
    }
    
    // Apply inline skinparams from source (these override base style)
    style = style.merge_skinparams(source);
    
    render_diagram_styled(source, &style)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sequence_diagram() {
        let source = "@startuml\nparticipant A\nA -> B: msg\n@enduml";
        assert_eq!(detect_diagram_type(source), DiagramType::Sequence);
    }

    #[test]
    fn test_detect_class_diagram() {
        let source = "@startuml\nclass Foo {}\n@enduml";
        assert_eq!(detect_diagram_type(source), DiagramType::Class);
    }

    #[test]
    fn test_render_sequence() {
        let source = "@startuml\nA -> B: hello\n@enduml";
        let svg = render_diagram(source);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("hello"));
    }

    #[test]
    fn test_render_class() {
        let source = "@startuml\nclass Engine {}\n@enduml";
        let svg = render_diagram(source);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Engine"));
    }
}
