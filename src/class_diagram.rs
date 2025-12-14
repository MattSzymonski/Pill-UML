//! Class diagram parser and renderer.
//!
//! Supports PlantUML class diagram syntax including:
//! - Classes, interfaces, abstract classes, enums
//! - Fields and methods with visibility modifiers
//! - Relationships: inheritance, realization, composition, aggregation, association

use crate::common::{escape_xml, DiagramStyle, SvgBuilder};
use std::collections::HashMap;

// ============================================================================
// Data Types
// ============================================================================

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
}

impl Visibility {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Public),
            '-' => Some(Self::Private),
            '#' => Some(Self::Protected),
            '~' => Some(Self::Package),
            _ => None,
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            Self::Public => "+",
            Self::Private => "-",
            Self::Protected => "#",
            Self::Package => "~",
        }
    }
}

/// A field in a class
#[derive(Debug, Clone)]
pub struct Field {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub field_type: Option<String>,
    pub is_static: bool,
}

/// A method in a class
#[derive(Debug, Clone)]
pub struct Method {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub params: String,
    pub return_type: Option<String>,
    pub is_static: bool,
    pub is_abstract: bool,
}

/// Type of class-like element
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClassType {
    Class,
    Interface,
    Abstract,
    Enum,
}

/// A class definition
#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub class_type: ClassType,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub stereotype: Option<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Relationship type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationType {
    Inheritance,   // --|>
    Realization,   // ..|>
    Composition,   // *--
    Aggregation,   // o--
    Association,   // --
    Dependency,    // ..>
    DirectedAssoc, // -->
}

/// A relationship between classes
#[derive(Debug, Clone)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub rel_type: RelationType,
    pub label: Option<String>,
}

/// Parsed class diagram
#[derive(Debug, Clone)]
pub struct ClassDiagram {
    pub classes: Vec<ClassDef>,
    pub relationships: Vec<Relationship>,
}

// ============================================================================
// Parser
// ============================================================================

struct Parser {
    classes: HashMap<String, ClassDef>,
    relationships: Vec<Relationship>,
    current_class: Option<String>,
}

impl Parser {
    fn new() -> Self {
        Self {
            classes: HashMap::new(),
            relationships: Vec::new(),
            current_class: None,
        }
    }

    fn parse(mut self, source: &str) -> ClassDiagram {
        let mut in_diagram = false;

        for line in source.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with("//") || line.starts_with("skinparam") {
                continue;
            }

            if line.starts_with("@start_uml") {
                in_diagram = true;
                continue;
            }
            if line.starts_with("@end_uml") {
                break;
            }

            if in_diagram {
                self.parse_line(line);
            }
        }

        ClassDiagram {
            classes: self.classes.into_values().collect(),
            relationships: self.relationships,
        }
    }

    fn parse_line(&mut self, line: &str) {
        // End of class body
        if line == "}" {
            self.current_class = None;
            return;
        }

        // Inside class body
        if let Some(ref name) = self.current_class.clone() {
            self.parse_member(&name, line);
            return;
        }

        // Class/interface declaration
        if line.starts_with("class ")
            || line.starts_with("interface ")
            || line.starts_with("abstract ")
            || line.starts_with("enum ")
        {
            self.parse_class_decl(line);
            return;
        }

        // Relationship
        self.try_parse_relationship(line);
    }

    fn parse_class_decl(&mut self, line: &str) {
        let (class_type, rest) = if line.starts_with("interface ") {
            (ClassType::Interface, &line[10..])
        } else if line.starts_with("abstract class ") {
            (ClassType::Abstract, &line[15..])
        } else if line.starts_with("abstract ") {
            (ClassType::Abstract, &line[9..])
        } else if line.starts_with("enum ") {
            (ClassType::Enum, &line[5..])
        } else if line.starts_with("class ") {
            (ClassType::Class, &line[6..])
        } else {
            return;
        };

        let rest = rest.trim();
        let has_body = rest.ends_with('{');
        let name_part = if has_body {
            rest[..rest.len() - 1].trim()
        } else {
            rest
        };

        // Parse stereotype <<...>>
        let (name, stereotype) = if let Some(start) = name_part.find("<<") {
            if let Some(end) = name_part.find(">>") {
                (
                    name_part[..start].trim().to_string(),
                    Some(name_part[start + 2..end].trim().to_string()),
                )
            } else {
                (name_part.to_string(), None)
            }
        } else {
            (name_part.to_string(), None)
        };

        self.classes.insert(
            name.clone(),
            ClassDef {
                name: name.clone(),
                class_type,
                fields: Vec::new(),
                methods: Vec::new(),
                stereotype,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        );

        if has_body {
            self.current_class = Some(name);
        }
    }

    fn parse_member(&mut self, class_name: &str, line: &str) {
        let line = line.trim();
        if line.is_empty() || line == "{" {
            return;
        }

        // Check visibility prefix
        let (visibility, rest) = if let Some(first) = line.chars().next() {
            if let Some(vis) = Visibility::from_char(first) {
                (Some(vis), line[1..].trim())
            } else {
                (None, line)
            }
        } else {
            (None, line)
        };

        // Method has parentheses
        if rest.contains('(') {
            self.parse_method(class_name, visibility, rest);
        } else {
            self.parse_field(class_name, visibility, rest);
        }
    }

    fn parse_field(&mut self, class_name: &str, visibility: Option<Visibility>, rest: &str) {
        let is_static = rest.contains("{static}") || rest.contains("{classifier}");
        let rest = rest.replace("{static}", "").replace("{classifier}", "");
        let rest = rest.trim();

        let (name, field_type) = if let Some(pos) = rest.find(':') {
            (
                rest[..pos].trim().to_string(),
                Some(rest[pos + 1..].trim().to_string()),
            )
        } else {
            (rest.to_string(), None)
        };

        if let Some(class) = self.classes.get_mut(class_name) {
            class.fields.push(Field {
                visibility,
                name,
                field_type,
                is_static,
            });
        }
    }

    fn parse_method(&mut self, class_name: &str, visibility: Option<Visibility>, rest: &str) {
        let is_static = rest.contains("{static}") || rest.contains("{classifier}");
        let is_abstract = rest.contains("{abstract}");
        let rest = rest
            .replace("{static}", "")
            .replace("{classifier}", "")
            .replace("{abstract}", "");
        let rest = rest.trim();

        if let (Some(paren_start), Some(paren_end)) = (rest.find('('), rest.find(')')) {
            let name = rest[..paren_start].trim().to_string();
            let params = rest[paren_start + 1..paren_end].trim().to_string();
            let return_type = rest[paren_end..]
                .find(':')
                .map(|pos| rest[paren_end + pos + 1..].trim().to_string());

            if let Some(class) = self.classes.get_mut(class_name) {
                class.methods.push(Method {
                    visibility,
                    name,
                    params,
                    return_type,
                    is_static,
                    is_abstract,
                });
            }
        }
    }

    fn try_parse_relationship(&mut self, line: &str) {
        let patterns = [
            ("--|>", RelationType::Inheritance),
            ("<|--", RelationType::Inheritance),
            ("..|>", RelationType::Realization),
            ("<|..", RelationType::Realization),
            ("*--", RelationType::Composition),
            ("--*", RelationType::Composition),
            ("o--", RelationType::Aggregation),
            ("--o", RelationType::Aggregation),
            ("..>", RelationType::Dependency),
            ("<..", RelationType::Dependency),
            ("-->", RelationType::DirectedAssoc),
            ("<--", RelationType::DirectedAssoc),
            ("--", RelationType::Association),
        ];

        for (pattern, rel_type) in patterns {
            if let Some(pos) = line.find(pattern) {
                let left = line[..pos].trim();
                let right_part = line[pos + pattern.len()..].trim();

                let (right, label) = if let Some(colon) = right_part.find(':') {
                    (
                        right_part[..colon].trim(),
                        Some(right_part[colon + 1..].trim().to_string()),
                    )
                } else {
                    (right_part, None)
                };

                if left.is_empty() || right.is_empty() {
                    continue;
                }

                let (from, to) = if pattern.starts_with('<') {
                    (right.to_string(), left.to_string())
                } else {
                    (left.to_string(), right.to_string())
                };

                self.ensure_class(&from);
                self.ensure_class(&to);

                self.relationships.push(Relationship {
                    from,
                    to,
                    rel_type,
                    label,
                });
                return;
            }
        }
    }

    fn ensure_class(&mut self, name: &str) {
        if !self.classes.contains_key(name) {
            self.classes.insert(
                name.to_string(),
                ClassDef {
                    name: name.to_string(),
                    class_type: ClassType::Class,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    stereotype: None,
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
            );
        }
    }
}

// ============================================================================
// Layout Engine
// ============================================================================

impl ClassDiagram {
    fn layout(&mut self, style: &DiagramStyle) {
        self.calculate_dimensions(style);
        self.hierarchical_layout(style);
    }

    fn calculate_dimensions(&mut self, style: &DiagramStyle) {
        let compartment_height = 25.0;
        let field_height = 18.0;
        let min_width = 120.0;

        for class in &mut self.classes {
            let name_width = class.name.len() as f32 * style.char_width + style.padding * 2.0;
            let mut max_width = name_width;

            for field in &class.fields {
                let text =
                    format_member(field.visibility, &field.name, field.field_type.as_deref());
                max_width =
                    max_width.max(text.len() as f32 * style.char_width + style.padding * 2.0);
            }

            for method in &class.methods {
                let text = format_method_text(method);
                max_width =
                    max_width.max(text.len() as f32 * style.char_width + style.padding * 2.0);
            }

            class.width = max_width.max(min_width);

            let fields_h = if class.fields.is_empty() {
                0.0
            } else {
                class.fields.len() as f32 * field_height + style.padding
            };
            let methods_h = if class.methods.is_empty() {
                0.0
            } else {
                class.methods.len() as f32 * field_height + style.padding
            };

            class.height =
                (compartment_height + fields_h + methods_h).max(compartment_height * 2.0);
        }
    }

    fn hierarchical_layout(&mut self, style: &DiagramStyle) {
        if self.classes.is_empty() {
            return;
        }

        let names: Vec<String> = self.classes.iter().map(|c| c.name.clone()).collect();
        let mut layers: HashMap<String, usize> = HashMap::new();
        let mut children: HashMap<String, Vec<String>> = HashMap::new();

        // Build hierarchy from relationships
        for rel in &self.relationships {
            match rel.rel_type {
                RelationType::Inheritance | RelationType::Realization => {
                    children
                        .entry(rel.to.clone())
                        .or_default()
                        .push(rel.from.clone());
                }
                RelationType::Composition | RelationType::Aggregation => {
                    children
                        .entry(rel.from.clone())
                        .or_default()
                        .push(rel.to.clone());
                }
                _ => {}
            }
        }

        // Find roots and assign layers via BFS
        let roots: Vec<_> = names
            .iter()
            .filter(|n| {
                !self.relationships.iter().any(|r| {
                    (r.rel_type == RelationType::Inheritance
                        || r.rel_type == RelationType::Realization)
                        && &r.from == *n
                })
            })
            .cloned()
            .collect();

        let mut queue: Vec<(String, usize)> = if roots.is_empty() {
            vec![(names[0].clone(), 0)]
        } else {
            roots.into_iter().map(|r| (r, 0)).collect()
        };

        while let Some((name, layer)) = queue.pop() {
            if layers.contains_key(&name) {
                continue;
            }
            layers.insert(name.clone(), layer);

            if let Some(kids) = children.get(&name) {
                for kid in kids {
                    if !layers.contains_key(kid) {
                        queue.push((kid.clone(), layer + 1));
                    }
                }
            }
        }

        // Assign remaining classes
        let max_layer = layers.values().copied().max().unwrap_or(0);
        for name in &names {
            layers.entry(name.clone()).or_insert(max_layer + 1);
        }

        // Group by layer
        let mut layer_groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, class) in self.classes.iter().enumerate() {
            let layer = *layers.get(&class.name).unwrap_or(&0);
            layer_groups.entry(layer).or_default().push(i);
        }

        // Position classes
        let mut layer_list: Vec<_> = layer_groups.keys().copied().collect();
        layer_list.sort();

        let mut current_y = style.margin;
        for layer in layer_list {
            let indices = layer_groups.get(&layer).unwrap();
            let max_height = indices
                .iter()
                .map(|&i| self.classes[i].height)
                .fold(0.0f32, f32::max);

            let mut current_x = style.margin;
            for &idx in indices {
                self.classes[idx].x = current_x;
                self.classes[idx].y = current_y;
                current_x += self.classes[idx].width + style.spacing_x;
            }

            current_y += max_height + style.spacing_y;
        }
    }

    fn bounds(&self, style: &DiagramStyle) -> (f32, f32) {
        let max_x = self
            .classes
            .iter()
            .map(|c| c.x + c.width)
            .fold(0.0f32, f32::max);
        let max_y = self
            .classes
            .iter()
            .map(|c| c.y + c.height)
            .fold(0.0f32, f32::max);
        (max_x + style.margin, max_y + style.margin)
    }
}

fn format_member(vis: Option<Visibility>, name: &str, typ: Option<&str>) -> String {
    let v = vis.map(|v| v.symbol()).unwrap_or("");
    match typ {
        Some(t) => format!("{}{}: {}", v, name, t),
        None => format!("{}{}", v, name),
    }
}

fn format_method_text(m: &Method) -> String {
    let v = m.visibility.map(|v| v.symbol()).unwrap_or("");
    match &m.return_type {
        Some(t) => format!("{}{}({}): {}", v, m.name, m.params, t),
        None => format!("{}{}({})", v, m.name, m.params),
    }
}

// ============================================================================
// Renderer
// ============================================================================

/// Render diagram with default behavior (no file CSS)
pub fn render(source: &str, style: &DiagramStyle) -> String {
    render_with_file_css(source, style, None)
}

/// Render diagram with optional file CSS layer
pub fn render_with_file_css(source: &str, style: &DiagramStyle, file_css: Option<&str>) -> String {
    let mut diagram = Parser::new().parse(source);
    diagram.layout(style);

    let (width, height) = diagram.bounds(style);
    let inline_css = crate::common::extract_custom_css(source);
    let mut svg = SvgBuilder::new(width, height, style, file_css, inline_css.as_deref());

    // Build defs section with markers and shadow filters
    let mut defs = String::from("<defs>\n");

    // Check for shadows on each class type and create filters
    let class_types = [
        ("class", "class-shadow"),
        ("interface", "interface-shadow"),
        ("abstract-class", "abstract-class-shadow"),
        ("enum", "enum-shadow"),
    ];
    for (class_name, filter_id) in &class_types {
        if svg.has_shadow(class_name) {
            let dx = svg.css_prop_or(class_name, "shadow-dx", 0.0);
            let dy = svg.css_prop_or(class_name, "shadow-dy", 0.0);
            let blur = svg.css_prop_or(class_name, "shadow-blur", 0.0);
            let opacity = svg.css_prop_or(class_name, "shadow-opacity", 0.3);
            defs.push_str(&format!(
                r#"<filter id="{}" x="-50%" y="-50%" width="200%" height="200%">
<feDropShadow dx="{}" dy="{}" stdDeviation="{}" flood-opacity="{}"/>
</filter>
"#,
                filter_id, dx, dy, blur, opacity
            ));
        }
    }

    // Markers for arrows with CSS classes
    defs.push_str(
        r#"<marker id="cls-triangle" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="10" markerHeight="10" orient="auto-start-reverse">
<path d="M 0 0 L 10 5 L 0 10 z" class="marker-triangle"/>
</marker>
<marker id="cls-arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
<path d="M 0 0 L 10 5 L 0 10 z" class="marker-arrow"/>
</marker>
<marker id="cls-diamond-filled" viewBox="0 0 12 12" refX="12" refY="6" markerWidth="12" markerHeight="12" orient="auto-start-reverse">
<path d="M 0 6 L 6 0 L 12 6 L 6 12 z" class="marker-diamond-filled"/>
</marker>
<marker id="cls-diamond-empty" viewBox="0 0 12 12" refX="12" refY="6" markerWidth="12" markerHeight="12" orient="auto-start-reverse">
<path d="M 0 6 L 6 0 L 12 6 L 6 12 z" class="marker-diamond-empty"/>
</marker>
"#);
    defs.push_str("</defs>");
    svg.push(&defs);

    // Render relationships first (behind classes)
    for rel in &diagram.relationships {
        render_relationship(&mut svg, &diagram, rel, style);
    }

    // Render classes
    for class in &diagram.classes {
        render_class(&mut svg, class, style);
    }

    svg.finish()
}

fn render_class(svg: &mut SvgBuilder, class: &ClassDef, style: &DiagramStyle) {
    let compartment_height = 25.0;
    let field_height = 18.0;

    // Determine class CSS based on type
    let (box_class, filter_id) = match class.class_type {
        ClassType::Interface => ("interface", "interface-shadow"),
        ClassType::Abstract => ("abstract-class", "abstract-class-shadow"),
        ClassType::Enum => ("enum", "enum-shadow"),
        ClassType::Class => ("class", "class-shadow"),
    };

    // Get border radius from CSS custom properties
    let rx = svg.css_prop_or(box_class, "rx", 0.0);
    let ry = svg.css_prop_or(box_class, "ry", 0.0);

    // Apply shadow filter if defined
    let filter = if svg.has_shadow(box_class) {
        Some(filter_id)
    } else {
        None
    };

    // Main box with optional rounded corners and shadow
    svg.rect_rounded_class_filtered(
        class.x,
        class.y,
        class.width,
        class.height,
        rx,
        ry,
        box_class,
        filter,
    );

    let mut y = class.y;

    // Stereotype
    if let Some(ref stereo) = class.stereotype {
        svg.text_class(
            class.x + class.width / 2.0,
            y + 12.0,
            &format!("<<{}>>", stereo),
            "class-stereotype",
        );
        y += 10.0;
    }

    // Name (italic for interface/abstract)
    let name_class = match class.class_type {
        ClassType::Interface => "interface-name",
        ClassType::Abstract => "abstract-class-name",
        _ => "class-name",
    };
    svg.text_class(
        class.x + class.width / 2.0,
        y + compartment_height / 2.0 + 4.0,
        &class.name,
        name_class,
    );

    y = class.y + compartment_height;

    // Separator class based on type
    let separator_class = match class.class_type {
        ClassType::Interface => "interface-separator",
        ClassType::Abstract => "abstract-class-separator",
        _ => "class-separator",
    };

    // Separator after header
    svg.line_class(class.x, y, class.x + class.width, y, separator_class);

    // Fields (interfaces don't have fields, but we handle it gracefully)
    if !class.fields.is_empty() {
        y += 4.0;
        for field in &class.fields {
            y += field_height;
            let text = format_member(field.visibility, &field.name, field.field_type.as_deref());
            let field_class = match (class.class_type, field.is_static) {
                (ClassType::Abstract, true) => {
                    "abstract-class-field-name abstract-class-field-name-static"
                }
                (ClassType::Abstract, false) => "abstract-class-field-name",
                (_, true) => "class-field-name class-field-name-static",
                (_, false) => "class-field-name",
            };
            svg.push(&format!(
                r#"<text x="{}" y="{}" class="{}">{}</text>"#,
                class.x + style.padding,
                y,
                field_class,
                escape_xml(&text)
            ));
        }
        y += 4.0;
        svg.line_class(class.x, y, class.x + class.width, y, separator_class);
    }

    // Methods
    if !class.methods.is_empty() {
        y += 4.0;
        for method in &class.methods {
            y += field_height;
            let text = format_method_text(method);
            let method_class = match class.class_type {
                ClassType::Interface => {
                    if method.is_static {
                        "interface-method-name interface-method-name-static"
                    } else {
                        "interface-method-name"
                    }
                }
                ClassType::Abstract => {
                    if method.is_static {
                        "abstract-class-method-name abstract-class-method-name-static"
                    } else if method.is_abstract {
                        "abstract-class-method-name abstract-class-method-name-abstract"
                    } else {
                        "abstract-class-method-name"
                    }
                }
                _ => {
                    if method.is_static {
                        "class-method-name class-method-name-static"
                    } else if method.is_abstract {
                        "class-method-name class-method-name-abstract"
                    } else {
                        "class-method-name"
                    }
                }
            };
            svg.push(&format!(
                r#"<text x="{}" y="{}" class="{}">{}</text>"#,
                class.x + style.padding,
                y,
                method_class,
                escape_xml(&text)
            ));
        }
    }
}

fn render_relationship(
    svg: &mut SvgBuilder,
    diagram: &ClassDiagram,
    rel: &Relationship,
    _style: &DiagramStyle,
) {
    let from = diagram.classes.iter().find(|c| c.name == rel.from);
    let to = diagram.classes.iter().find(|c| c.name == rel.to);

    let (from, to) = match (from, to) {
        (Some(f), Some(t)) => (f, t),
        _ => return,
    };

    let (dashed, marker_start, marker_end) = match rel.rel_type {
        RelationType::Inheritance => (false, "", "url(#cls-triangle)"),
        RelationType::Realization => (true, "", "url(#cls-triangle)"),
        RelationType::Composition => (false, "url(#cls-diamond-filled)", ""),
        RelationType::Aggregation => (false, "url(#cls-diamond-empty)", ""),
        RelationType::Association => (false, "", ""),
        RelationType::Dependency => (true, "", "url(#cls-arrow)"),
        RelationType::DirectedAssoc => (false, "", "url(#cls-arrow)"),
    };

    let points = calculate_path(from, to, rel.rel_type);

    if !points.is_empty() {
        let points_str: String = points
            .iter()
            .map(|(x, y)| format!("{},{}", x, y))
            .collect::<Vec<_>>()
            .join(" ");
        let class = if dashed {
            "relationship relationship-dashed"
        } else {
            "relationship"
        };
        let ms = if marker_start.is_empty() {
            String::new()
        } else {
            format!(r#" marker-start="{}""#, marker_start)
        };
        let me = if marker_end.is_empty() {
            String::new()
        } else {
            format!(r#" marker-end="{}""#, marker_end)
        };

        svg.push(&format!(
            r#"<polyline points="{}" class="{}"{}{}/>"#,
            points_str, class, ms, me
        ));

        if let Some(ref label) = rel.label {
            let mid = points.len() / 2;
            let (mx, my) = points.get(mid).copied().unwrap_or((0.0, 0.0));
            svg.text_class(mx, my - 5.0, label, "relationship-label");
        }
    }
}

fn calculate_path(from: &ClassDef, to: &ClassDef, rel_type: RelationType) -> Vec<(f32, f32)> {
    let from_cx = from.x + from.width / 2.0;
    let from_cy = from.y + from.height / 2.0;
    let to_cx = to.x + to.width / 2.0;
    let to_cy = to.y + to.height / 2.0;

    let dx = to_cx - from_cx;
    let dy = to_cy - from_cy;
    let route_margin = 15.0;

    let prefer_vertical = matches!(
        rel_type,
        RelationType::Inheritance | RelationType::Realization
    );

    if prefer_vertical && dy.abs() > 20.0 {
        if dy > 0.0 {
            // From is above To
            let (sx, sy) = (from_cx, from.y + from.height);
            let (ex, ey) = (to_cx, to.y);
            if (sx - ex).abs() < 10.0 {
                return vec![(sx, sy), (ex, ey)];
            }
            let mid_y = (sy + ey) / 2.0;
            return vec![(sx, sy), (sx, mid_y), (ex, mid_y), (ex, ey)];
        } else {
            let (sx, sy) = (from_cx, from.y);
            let (ex, ey) = (to_cx, to.y + to.height);
            if (sx - ex).abs() < 10.0 {
                return vec![(sx, sy), (ex, ey)];
            }
            let mid_y = (sy + ey) / 2.0;
            return vec![(sx, sy), (sx, mid_y), (ex, mid_y), (ex, ey)];
        }
    }

    // Horizontal routing
    if dx.abs() > dy.abs() || !prefer_vertical {
        if dx > 0.0 {
            let (sx, sy) = (from.x + from.width, from_cy);
            let (ex, ey) = (to.x, to_cy);
            if ex - sx > route_margin * 2.0 {
                let mid_x = (sx + ex) / 2.0;
                return vec![(sx, sy), (mid_x, sy), (mid_x, ey), (ex, ey)];
            }
            let route_y = if from_cy > to_cy {
                from.y.min(to.y) - route_margin
            } else {
                (from.y + from.height).max(to.y + to.height) + route_margin
            };
            return vec![
                (sx, sy),
                (sx + route_margin, sy),
                (sx + route_margin, route_y),
                (ex - route_margin, route_y),
                (ex - route_margin, ey),
                (ex, ey),
            ];
        } else {
            let (sx, sy) = (from.x, from_cy);
            let (ex, ey) = (to.x + to.width, to_cy);
            if sx - ex > route_margin * 2.0 {
                let mid_x = (sx + ex) / 2.0;
                return vec![(sx, sy), (mid_x, sy), (mid_x, ey), (ex, ey)];
            }
            let route_y = if from_cy > to_cy {
                from.y.min(to.y) - route_margin
            } else {
                (from.y + from.height).max(to.y + to.height) + route_margin
            };
            return vec![
                (sx, sy),
                (sx - route_margin, sy),
                (sx - route_margin, route_y),
                (ex + route_margin, route_y),
                (ex + route_margin, ey),
                (ex, ey),
            ];
        }
    }

    // Vertical
    if dy > 0.0 {
        let (sx, sy) = (from_cx, from.y + from.height);
        let (ex, ey) = (to_cx, to.y);
        let mid_y = (sy + ey) / 2.0;
        vec![(sx, sy), (sx, mid_y), (ex, mid_y), (ex, ey)]
    } else {
        let (sx, sy) = (from_cx, from.y);
        let (ex, ey) = (to_cx, to.y + to.height);
        let mid_y = (sy + ey) / 2.0;
        vec![(sx, sy), (sx, mid_y), (ex, mid_y), (ex, ey)]
    }
}

/// Check if source looks like a class diagram
pub fn is_class_diagram(source: &str) -> bool {
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("class ")
            || line.starts_with("interface ")
            || line.starts_with("abstract ")
            || line.starts_with("enum ")
            || line.contains("--|>")
            || line.contains("<|--")
            || line.contains("..|>")
            || line.contains("<|..")
            || line.contains("*--")
            || line.contains("--*")
            || line.contains("o--")
            || line.contains("--o")
        {
            return true;
        }
        if line.starts_with("participant ")
            || line.starts_with("actor ")
            || (line.contains("->") && line.contains(':') && !line.contains("--|>"))
        {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_class() {
        let source = "@start_uml\nclass Foo {\n- x: i32\n+ bar(): void\n}\n@end_uml";
        let diagram = Parser::new().parse(source);
        assert_eq!(diagram.classes.len(), 1);
        assert_eq!(diagram.classes[0].fields.len(), 1);
        assert_eq!(diagram.classes[0].methods.len(), 1);
    }

    #[test]
    fn test_parse_relationship() {
        let source = "@start_uml\nA --|> B\n@end_uml";
        let diagram = Parser::new().parse(source);
        assert_eq!(diagram.relationships.len(), 1);
        assert_eq!(diagram.relationships[0].rel_type, RelationType::Inheritance);
    }

    #[test]
    fn test_is_class_diagram() {
        assert!(is_class_diagram("class Foo {}"));
        assert!(!is_class_diagram("participant A\nA -> B: msg"));
    }
}
