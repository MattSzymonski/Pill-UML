//! Common types, styling, and utilities shared across diagram types.

use std::sync::LazyLock;

// ============================================================================
// Default CSS Styles
// ============================================================================

/// Embedded default CSS styles
pub const DEFAULT_STYLES_CSS: &str = include_str!("./default_styles.css");

/// Extract custom CSS from @start_style / @end_style block in source
pub fn extract_custom_css(source: &str) -> Option<String> {
    let mut in_style = false;
    let mut css_lines = Vec::new();
    
    for line in source.lines() {
        let trimmed = line.trim();
        
        if trimmed == "@start_style" {
            in_style = true;
            continue;
        }
        
        if trimmed == "@end_style" {
            break;
        }
        
        if in_style {
            // Skip comments
            if !trimmed.starts_with("//") {
                css_lines.push(line);
            }
        }
    }
    
    if css_lines.is_empty() {
        None
    } else {
        Some(css_lines.join("\n"))
    }
}

// ============================================================================
// Diagram Types
// ============================================================================

/// Supported diagram types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramType {
    Sequence,
    Class,
}

// ============================================================================
// Styling
// ============================================================================

/// Unified style configuration for all diagram types
#[derive(Debug, Clone)]
pub struct DiagramStyle {
    // Colors
    pub background_color: String,
    pub font_color: String,
    pub border_color: String,
    pub arrow_color: String,
    pub arrow_thickness: f32,

    // Sequence diagram specific
    pub participant_bg_color: String,
    pub participant_border_color: String,
    pub lifeline_color: String,
    pub alt_bg_color: String,
    pub alt_border_color: String,

    // Class diagram specific
    pub class_bg_color: String,
    pub class_border_color: String,
    pub interface_bg_color: String,

    // Dimensions
    pub margin: f32,
    pub padding: f32,
    pub font_size: f32,
    pub char_width: f32,
    pub spacing_x: f32,
    pub spacing_y: f32,

    // Fonts
    pub font_family: String,
    pub embed_font: bool,
}

impl Default for DiagramStyle {
    fn default() -> Self {
        Self {
            background_color: "#FFFFFF".into(),
            font_color: "#333333".into(),
            border_color: "#333333".into(),
            arrow_color: "#333333".into(),
            arrow_thickness: 1.5,

            participant_bg_color: "#F0F0F0".into(),
            participant_border_color: "#333333".into(),
            lifeline_color: "#666666".into(),
            alt_bg_color: "#FAFAFA".into(),
            alt_border_color: "#999999".into(),

            class_bg_color: "#F0F0F0".into(),
            class_border_color: "#333333".into(),
            interface_bg_color: "#E8F4E8".into(),

            margin: 30.0,
            padding: 10.0,
            font_size: 12.0,
            char_width: 7.0,
            spacing_x: 60.0,
            spacing_y: 80.0,

            font_family: format!("'{}', monospace", FONT_FAMILY),
            embed_font: true,
        }
    }
}

impl DiagramStyle {
    /// Create style with custom font family
    pub fn with_font_family(mut self, family: &str) -> Self {
        self.font_family = family.to_string();
        self.embed_font = false;
        self
    }

    /// Create style with custom background color
    pub fn with_background_color(mut self, color: &str) -> Self {
        self.background_color = color.to_string();
        self
    }

    /// Create style with custom font color
    pub fn with_font_color(mut self, color: &str) -> Self {
        self.font_color = color.to_string();
        self
    }
}

// ============================================================================
// Font Embedding
// ============================================================================

/// Embedded font file bytes
const FONT_BYTES: &[u8] = include_bytes!("./inter.ttf");

/// Base64-encoded font (computed lazily)
static FONT_BASE64: LazyLock<String> = LazyLock::new(|| base64_encode(FONT_BYTES));

/// Font family name for CSS
pub const FONT_FAMILY: &str = "MomoTrust";

/// Generate SVG font embedding CSS
pub fn font_style_defs() -> String {
    format!(
        r#"<style type="text/css">
@font-face {{
    font-family: '{}';
    src: url('data:font/truetype;base64,{}') format('truetype');
}}
</style>"#,
        FONT_FAMILY,
        FONT_BASE64.as_str()
    )
}

/// Simple base64 encoder
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).map(|&b| b as u32).unwrap_or(0);
        let b2 = chunk.get(2).map(|&b| b as u32).unwrap_or(0);

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        result.push(if chunk.len() > 1 {
            ALPHABET[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            ALPHABET[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }

    result
}

// ============================================================================
// SVG Utilities
// ============================================================================

/// SVG builder helper
pub struct SvgBuilder {
    output: String,
}

impl SvgBuilder {
    /// Create new SVG builder with optional custom CSS overrides
    pub fn new(width: f32, height: f32, style: &DiagramStyle, custom_css: Option<&str>) -> Self {
        let mut output = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
            width, height
        );

        // Embed default CSS styles
        output.push_str("<style type=\"text/css\">\n");
        output.push_str(DEFAULT_STYLES_CSS);
        
        // Append custom CSS overrides if provided
        if let Some(css) = custom_css {
            output.push_str("\n/* Custom style overrides */\n");
            output.push_str(css);
        }
        output.push_str("\n</style>");

        // Embed font if enabled
        if style.embed_font {
            output.push_str(&font_style_defs());
        }

        // Background
        output.push_str(r#"<rect width="100%" height="100%" class="diagram-background"/>"#);

        Self { output }
    }

    pub fn push(&mut self, content: &str) {
        self.output.push_str(content);
    }
    
    // ========================================================================
    // CSS class-based methods
    // ========================================================================
    
    /// Draw a rectangle with CSS class
    pub fn rect_class(&mut self, x: f32, y: f32, w: f32, h: f32, class: &str) {
        self.output.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" class="{}"/>"#,
            x, y, w, h, class
        ));
    }
    
    /// Draw a line with CSS class
    pub fn line_class(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, class: &str) {
        self.output.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="{}"/>"#,
            x1, y1, x2, y2, class
        ));
    }
    
    /// Draw text with CSS class
    pub fn text_class(&mut self, x: f32, y: f32, content: &str, class: &str) {
        self.output.push_str(&format!(
            r#"<text x="{}" y="{}" class="{}">{}</text>"#,
            x, y, class, escape_xml(content)
        ));
    }
    
    /// Draw a polyline with CSS class
    pub fn polyline_class(&mut self, points: &[(f32, f32)], class: &str, marker_end: &str) {
        let points_str: String = points
            .iter()
            .map(|(x, y)| format!("{},{}", x, y))
            .collect::<Vec<_>>()
            .join(" ");

        let marker = if marker_end.is_empty() {
            String::new()
        } else {
            format!(r#" marker-end="{}""#, marker_end)
        };

        self.output.push_str(&format!(
            r#"<polyline points="{}" class="{}"{}/>"#,
            points_str, class, marker
        ));
    }
    
    /// Draw a polygon with CSS class
    pub fn polygon_class(&mut self, points: &[(f32, f32)], class: &str) {
        let points_str: String = points
            .iter()
            .map(|(x, y)| format!("{},{}", x, y))
            .collect::<Vec<_>>()
            .join(" ");

        self.output.push_str(&format!(
            r#"<polygon points="{}" class="{}"/>"#,
            points_str, class
        ));
    }

    // ========================================================================
    // Legacy inline style methods (kept for compatibility)
    // ========================================================================

    #[allow(dead_code)]
    pub fn line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: &str,
        width: f32,
        dashed: bool,
    ) {
        let dash = if dashed {
            r#" stroke-dasharray="5,5""#
        } else {
            ""
        };
        self.output.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"{}/>"#,
            x1, y1, x2, y2, color, width, dash
        ));
    }

    #[allow(dead_code)]
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: &str, stroke: &str) {
        self.output.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="1"/>"#,
            x, y, w, h, fill, stroke
        ));
    }

    #[allow(dead_code)]
    pub fn text(&mut self, x: f32, y: f32, content: &str, style: &DiagramStyle) {
        self.output.push_str(&format!(
            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
            x,
            y,
            style.font_family,
            style.font_size,
            style.font_color,
            escape_xml(content)
        ));
    }

    #[allow(dead_code)]
    pub fn text_centered(
        &mut self,
        x: f32,
        y: f32,
        content: &str,
        style: &DiagramStyle,
        bold: bool,
    ) {
        let weight = if bold { r#" font-weight="bold""# } else { "" };
        self.output.push_str(&format!(
            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}" text-anchor="middle"{}>{}</text>"#,
            x, y, style.font_family, style.font_size, style.font_color, weight, escape_xml(content)
        ));
    }

    #[allow(dead_code)]
    pub fn polyline(
        &mut self,
        points: &[(f32, f32)],
        color: &str,
        width: f32,
        dashed: bool,
        marker_end: &str,
    ) {
        let points_str: String = points
            .iter()
            .map(|(x, y)| format!("{},{}", x, y))
            .collect::<Vec<_>>()
            .join(" ");

        let dash = if dashed {
            format!(r#" stroke-dasharray="5,5""#)
        } else {
            String::new()
        };
        let marker = if marker_end.is_empty() {
            String::new()
        } else {
            format!(r#" marker-end="{}""#, marker_end)
        };

        self.output.push_str(&format!(
            r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="{}"{}{}/>"#,
            points_str, color, width, dash, marker
        ));
    }

    pub fn finish(mut self) -> String {
        self.output.push_str("</svg>");
        self.output
    }

    #[allow(dead_code)]
    pub fn raw(&self) -> &str {
        &self.output
    }
}

/// Escape XML special characters
pub fn escape_xml(text: &str) -> String {
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
    fn test_base64() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
        assert_eq!(base64_encode(b"ABC"), "QUJD");
    }

    #[test]
    fn test_style_builder() {
        let style = DiagramStyle::default()
            .with_background_color("#000000")
            .with_font_color("#FFFFFF");

        assert_eq!(style.background_color, "#000000");
        assert_eq!(style.font_color, "#FFFFFF");
    }

    #[test]
    fn test_extract_custom_css() {
        let source = "@start_style\n.test { fill: red; }\n@end_style\n@start_uml\n@end_uml";
        let css = extract_custom_css(source);
        assert!(css.is_some());
        assert!(css.unwrap().contains(".test { fill: red; }"));
    }
    
    #[test]
    fn test_extract_custom_css_none() {
        let source = "@start_uml\nA -> B: test\n@end_uml";
        let css = extract_custom_css(source);
        assert!(css.is_none());
    }
}
