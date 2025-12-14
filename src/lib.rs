//! # Pill UML
//!
//! A pure Rust diagram renderer that generates SVG output.
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
//! @start_uml
//! participant Client
//! participant Server
//! Client -> Server: Request
//! Server --> Client: Response
//! @end_uml
//! "#;
//!
//! let svg = render_diagram(source);
//! ```
//!
//! ## Builder Pattern with Style File
//!
//! You can use an external CSS file to override default styles:
//!
//! ```rust,ignore
//! use pill_uml::create_diagram;
//!
//! let svg = create_diagram(source)
//!     .with_style_file("path/to/styles.css")
//!     .render();
//! ```
//!
//! ## CSS Override Priority (lowest to highest)
//!
//! 1. Default styles (embedded in library)
//! 2. External style file (via `.with_style_file()`)
//! 3. Inline styles in `.pilluml` file (`@start_style`/`@end_style`)
//!
//! ## Custom Styling with CSS
//!
//! You can override default styles directly in your `.pilluml` file using
//! `@start_style` and `@end_style` blocks:
//!
//! ```text
//! @start_style
//! .participant { fill: #2d2d2d; stroke: #00ff88; }
//! .message { stroke: #00ccff; }
//! @end_style
//!
//! @start_uml
//! Client -> Server: Request
//! @end_uml
//! ```

mod class_diagram;
mod common;
mod sequence_diagram;

use std::fs;
use std::path::Path;

pub use class_diagram::{ClassDef, ClassDiagram, RelationType};
pub use common::{DiagramStyle, DiagramType, DEFAULT_STYLES_CSS};
pub use sequence_diagram::{ArrowStyle, Message, Participant, SequenceDiagram};

// ============================================================================
// Builder Pattern API
// ============================================================================

/// Builder for creating diagrams with optional style file
pub struct DiagramBuilder<'a> {
    source: &'a str,
    style: DiagramStyle,
    style_file_css: Option<String>,
}

impl<'a> DiagramBuilder<'a> {
    /// Create a new diagram builder with the given source
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            style: DiagramStyle::default(),
            style_file_css: None,
        }
    }

    /// Set an external CSS style file to override default styles.
    ///
    /// The style file CSS is applied after the default styles but before
    /// any inline `@start_style`/`@end_style` blocks in the source.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let svg = create_diagram(source)
    ///     .with_style_file("my_theme.css")
    ///     .render();
    /// ```
    pub fn with_style_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        match fs::read_to_string(path.as_ref()) {
            Ok(css) => self.style_file_css = Some(css),
            Err(e) => eprintln!("Warning: Could not read style file: {}", e),
        }
        self
    }

    /// Set CSS content directly (alternative to with_style_file)
    pub fn with_style_css(mut self, css: &str) -> Self {
        self.style_file_css = Some(css.to_string());
        self
    }

    /// Set a custom DiagramStyle
    pub fn with_diagram_style(mut self, style: DiagramStyle) -> Self {
        self.style = style;
        self
    }

    /// Render the diagram to SVG
    pub fn render(self) -> String {
        match detect_diagram_type(self.source) {
            DiagramType::Sequence => sequence_diagram::render_with_file_css(
                self.source,
                &self.style,
                self.style_file_css.as_deref(),
            ),
            DiagramType::Class => class_diagram::render_with_file_css(
                self.source,
                &self.style,
                self.style_file_css.as_deref(),
            ),
        }
    }
}

/// Create a diagram builder for the given source
///
/// # Example
///
/// ```rust,ignore
/// use pill_uml::create_diagram;
///
/// let svg = create_diagram(source)
///     .with_style_file("theme.css")
///     .render();
/// ```
pub fn create_diagram(source: &str) -> DiagramBuilder<'_> {
    DiagramBuilder::new(source)
}

/// Detect the diagram type from source
pub fn detect_diagram_type(source: &str) -> DiagramType {
    if class_diagram::is_class_diagram(source) {
        DiagramType::Class
    } else {
        DiagramType::Sequence
    }
}

/// Render a diagram to SVG with default styling
///
/// Automatically detects whether it's a sequence or class diagram.
/// Custom CSS can be embedded in the source using @start_style/@end_style blocks.
pub fn render_diagram(source: &str) -> String {
    render_diagram_styled(source, &DiagramStyle::default())
}

/// Render a diagram to SVG with custom DiagramStyle
pub fn render_diagram_styled(source: &str, style: &DiagramStyle) -> String {
    match detect_diagram_type(source) {
        DiagramType::Sequence => sequence_diagram::render(source, style),
        DiagramType::Class => class_diagram::render(source, style),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sequence_diagram() {
        let source = "@start_uml\nparticipant A\nA -> B: msg\n@end_uml";
        assert_eq!(detect_diagram_type(source), DiagramType::Sequence);
    }

    #[test]
    fn test_detect_class_diagram() {
        let source = "@start_uml\nclass Foo {}\n@end_uml";
        assert_eq!(detect_diagram_type(source), DiagramType::Class);
    }

    #[test]
    fn test_render_sequence() {
        let source = "@start_uml\nA -> B: hello\n@end_uml";
        let svg = render_diagram(source);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("hello"));
    }

    #[test]
    fn test_render_class() {
        let source = "@start_uml\nclass Engine {}\n@end_uml";
        let svg = render_diagram(source);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Engine"));
    }

    #[test]
    fn test_custom_css() {
        let source = "@start_style\n.participant { fill: #ff0000; }\n@end_style\n@start_uml\nA -> B: test\n@end_uml";
        let svg = render_diagram(source);
        assert!(svg.contains("fill: #ff0000"));
    }
}
