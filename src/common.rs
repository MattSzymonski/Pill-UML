//! Common types, styling, and utilities shared across diagram types.

use std::collections::HashMap;

// ============================================================================
// Default CSS Styles
// ============================================================================

/// Embedded default CSS styles
pub const DEFAULT_STYLES_CSS: &str = include_str!("./default_theme.css");

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

/// Extract CSS custom properties (--property: value) for a specific class
/// This allows controlling SVG attributes like rx/ry via CSS-like syntax
pub fn extract_css_property(css: &str, class: &str, property: &str) -> Option<f32> {
    // Find the class block
    let class_pattern = format!(".{}", class);
    let mut in_class = false;
    let mut brace_depth = 0;

    for line in css.lines() {
        let trimmed = line.trim();

        if trimmed.contains(&class_pattern) && trimmed.contains('{') {
            in_class = true;
            brace_depth = 1;
            continue;
        }

        if in_class {
            if trimmed.contains('{') {
                brace_depth += 1;
            }
            if trimmed.contains('}') {
                brace_depth -= 1;
                if brace_depth == 0 {
                    break;
                }
            }

            // Look for --property: value;
            let prop_pattern = format!("--{}:", property);
            if let Some(pos) = trimmed.find(&prop_pattern) {
                let value_start = pos + prop_pattern.len();
                let value_str = &trimmed[value_start..];
                // Extract number before ; or end of line
                let value_str = value_str.trim().trim_end_matches(';').trim();
                // Remove 'px' suffix if present
                let value_str = value_str.trim_end_matches("px");
                if let Ok(val) = value_str.parse::<f32>() {
                    return Some(val);
                }
            }
        }
    }

    None
}

/// Collected CSS custom properties for rendering
#[derive(Debug, Clone, Default)]
pub struct CssProperties {
    properties: HashMap<String, HashMap<String, f32>>,
}

impl CssProperties {
    /// Parse CSS and extract all custom properties (--name: value)
    pub fn from_css(css: &str) -> Self {
        let mut props = Self::default();
        props.parse_css(css);
        props
    }

    /// Parse and merge additional CSS
    pub fn merge_css(&mut self, css: &str) {
        self.parse_css(css);
    }

    fn parse_css(&mut self, css: &str) {
        let mut current_class: Option<String> = None;
        let mut brace_depth = 0;

        for line in css.lines() {
            let trimmed = line.trim();

            // Check for class selector
            if trimmed.starts_with('.') && trimmed.contains('{') {
                if let Some(class_end) = trimmed.find(|c| c == ' ' || c == '{') {
                    current_class = Some(trimmed[1..class_end].to_string());
                    brace_depth = 1;
                }
                continue;
            }

            if current_class.is_some() {
                if trimmed.contains('{') {
                    brace_depth += 1;
                }
                if trimmed.contains('}') {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        current_class = None;
                    }
                }

                // Parse --property: value;
                if let Some(pos) = trimmed.find("--") {
                    if let Some(colon_pos) = trimmed[pos..].find(':') {
                        let prop_name = trimmed[pos + 2..pos + colon_pos].trim().to_string();
                        let value_start = pos + colon_pos + 1;
                        let value_str = trimmed[value_start..].trim().trim_end_matches(';').trim();
                        let value_str = value_str.trim_end_matches("px");

                        if let Ok(val) = value_str.parse::<f32>() {
                            if let Some(ref class) = current_class {
                                self.properties
                                    .entry(class.clone())
                                    .or_default()
                                    .insert(prop_name, val);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get a property value for a class
    pub fn get(&self, class: &str, property: &str) -> Option<f32> {
        self.properties
            .get(class)
            .and_then(|m| m.get(property).copied())
    }

    /// Get a property value with a default
    pub fn get_or(&self, class: &str, property: &str, default: f32) -> f32 {
        self.get(class, property).unwrap_or(default)
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

            font_family: "'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif".into(),
        }
    }
}

impl DiagramStyle {
    /// Create style with custom font family
    pub fn with_font_family(mut self, family: &str) -> Self {
        self.font_family = family.to_string();
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
// SVG Utilities
// ============================================================================

/// SVG builder helper
pub struct SvgBuilder {
    output: String,
    css_props: CssProperties,
}

impl SvgBuilder {
    /// Create new SVG builder with optional CSS overrides
    ///
    /// CSS is layered in this order (lowest to highest priority):
    /// 1. Default styles (DEFAULT_STYLES_CSS)
    /// 2. File CSS (from external .css file)
    /// 3. Inline CSS (from @start_style/@end_style in source)
    pub fn new(
        width: f32,
        height: f32,
        _style: &DiagramStyle,
        file_css: Option<&str>,
        inline_css: Option<&str>,
    ) -> Self {
        // Parse CSS properties from all layers (in order of priority)
        let mut css_props = CssProperties::from_css(DEFAULT_STYLES_CSS);
        if let Some(css) = file_css {
            css_props.merge_css(css);
        }
        if let Some(css) = inline_css {
            css_props.merge_css(css);
        }

        let mut output = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
            width, height
        );

        // Embed default CSS styles
        output.push_str("<style type=\"text/css\">\n");
        output.push_str(DEFAULT_STYLES_CSS);

        // Append file CSS overrides if provided (middle layer)
        if let Some(css) = file_css {
            output.push_str("\n/* Style file overrides */\n");
            output.push_str(css);
        }

        // Append inline CSS overrides if provided (top layer)
        if let Some(css) = inline_css {
            output.push_str("\n/* Inline style overrides */\n");
            output.push_str(css);
        }
        output.push_str("\n</style>");

        // Background
        output.push_str(r#"<rect width="100%" height="100%" class="diagram-background"/>"#);

        Self { output, css_props }
    }

    /// Get a CSS custom property value (--name) for a class
    pub fn css_prop(&self, class: &str, property: &str) -> Option<f32> {
        self.css_props.get(class, property)
    }

    /// Get a CSS custom property value with default
    pub fn css_prop_or(&self, class: &str, property: &str, default: f32) -> f32 {
        self.css_props.get_or(class, property, default)
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

    /// Draw a rectangle with CSS class and rounded corners
    pub fn rect_rounded_class(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rx: f32,
        ry: f32,
        class: &str,
    ) {
        self.output.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" ry="{}" class="{}"/>"#,
            x, y, w, h, rx, ry, class
        ));
    }

    /// Draw a rectangle with CSS class, rounded corners, and optional filter
    pub fn rect_rounded_class_filtered(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rx: f32,
        ry: f32,
        class: &str,
        filter: Option<&str>,
    ) {
        let filter_attr = filter
            .map(|f| format!(r#" filter="url(#{})""#, f))
            .unwrap_or_default();
        self.output.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" ry="{}" class="{}"{}/>"#,
            x, y, w, h, rx, ry, class, filter_attr
        ));
    }

    /// Check if a shadow is defined for a class (any shadow property is non-zero)
    pub fn has_shadow(&self, class: &str) -> bool {
        let dx = self.css_prop_or(class, "shadow-dx", 0.0);
        let dy = self.css_prop_or(class, "shadow-dy", 0.0);
        let blur = self.css_prop_or(class, "shadow-blur", 0.0);
        dx != 0.0 || dy != 0.0 || blur != 0.0
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
            x,
            y,
            class,
            escape_xml(content)
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
