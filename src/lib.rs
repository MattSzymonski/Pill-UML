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

/// Builder for creating diagrams with optional style overrides
///
/// CSS styles are applied in this order (lowest to highest priority):
/// 1. Default styles (embedded in library)
/// 2. External styles added via `with_style()` or `with_style_file()` - in call order
/// 3. Inline styles in `.pilluml` file (`@start_style`/`@end_style`)
pub struct DiagramBuilder<'a> {
    source: &'a str,
    style: DiagramStyle,
    external_css: Vec<String>,
}

impl<'a> DiagramBuilder<'a> {
    /// Create a new diagram builder with the given source
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            style: DiagramStyle::default(),
            external_css: Vec::new(),
        }
    }

    /// Add CSS from a file to override default styles.
    ///
    /// Multiple calls accumulate CSS in order. Later calls override earlier ones.
    /// Inline `@start_style`/`@end_style` blocks in the source always have highest priority.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let svg = create_diagram(source)
    ///     .with_style_file("base_theme.css")
    ///     .with_style_file("overrides.css")  // overrides base_theme
    ///     .render();
    /// ```
    pub fn with_style_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        match fs::read_to_string(path.as_ref()) {
            Ok(css) => self.external_css.push(css),
            Err(e) => eprintln!("Warning: Could not read style file: {}", e),
        }
        self
    }

    /// Add CSS string to override default styles.
    ///
    /// Multiple calls accumulate CSS in order. Later calls override earlier ones.
    /// Inline `@start_style`/`@end_style` blocks in the source always have highest priority.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let svg = create_diagram(source)
    ///     .with_style(".participant { fill: #333; }")
    ///     .with_style_file("theme.css")  // overrides with_style
    ///     .render();
    /// ```
    pub fn with_style(mut self, css: &str) -> Self {
        self.external_css.push(css.to_string());
        self
    }

    /// Set a custom DiagramStyle for layout parameters
    pub fn with_diagram_style(mut self, style: DiagramStyle) -> Self {
        self.style = style;
        self
    }

    /// Render the diagram to SVG
    pub fn render(self) -> String {
        // Combine all external CSS into one string
        let combined_css = if self.external_css.is_empty() {
            None
        } else {
            Some(self.external_css.join("\n"))
        };

        match detect_diagram_type(self.source) {
            DiagramType::Sequence => sequence_diagram::render_with_file_css(
                self.source,
                &self.style,
                combined_css.as_deref(),
            ),
            DiagramType::Class => class_diagram::render_with_file_css(
                self.source,
                &self.style,
                combined_css.as_deref(),
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
