//! Sequence diagram parser and renderer.
//!
//! Supports PlantUML sequence diagram syntax including:
//! - Participants and actors
//! - Messages (solid, dashed, open arrows)
//! - Self-messages
//! - Alt/else blocks
//! - Dividers
//! - Notes

use crate::common::{DiagramStyle, SvgBuilder};
use std::collections::HashMap;

// ============================================================================
// Data Types
// ============================================================================

/// Arrow style for messages
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArrowStyle {
    Solid,      // ->
    Dashed,     // -->
    SolidOpen,  // ->>
    DashedOpen, // -->>
}

/// A participant in the sequence diagram
#[derive(Debug, Clone)]
pub struct Participant {
    pub name: String,
    pub order: i32,
    pub x: f32,
    pub width: f32,
}

/// A message between participants
#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub text: String,
    pub style: ArrowStyle,
}

/// Elements in a sequence diagram
#[derive(Debug, Clone)]
pub enum Element {
    Message(Message),
    Divider(String),
    AltStart(String),
    ElseBranch(Option<String>),
    AltEnd,
    Note { on: String, text: String },
}

/// Parsed sequence diagram
#[derive(Debug, Clone)]
pub struct SequenceDiagram {
    pub participants: Vec<Participant>,
    pub elements: Vec<Element>,
}

// ============================================================================
// Parser
// ============================================================================

struct Parser {
    participants: HashMap<String, Participant>,
    participant_order: i32,
    elements: Vec<Element>,
}

impl Parser {
    fn new() -> Self {
        Self {
            participants: HashMap::new(),
            participant_order: 0,
            elements: Vec::new(),
        }
    }

    fn parse(mut self, source: &str) -> SequenceDiagram {
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

        let mut participants: Vec<Participant> = self.participants.into_values().collect();
        participants.sort_by_key(|p| p.order);

        SequenceDiagram {
            participants,
            elements: self.elements,
        }
    }

    fn parse_line(&mut self, line: &str) {
        // Participant declaration
        if line.starts_with("participant ") {
            self.parse_participant(&line[12..]);
            return;
        }
        if line.starts_with("actor ") {
            self.parse_participant(&line[6..]);
            return;
        }

        // Divider: ...text...
        if line.starts_with("...") && line.ends_with("...") {
            let text = line.trim_matches('.').trim().to_string();
            self.elements.push(Element::Divider(text));
            return;
        }

        // Alt/else/end
        if line.starts_with("alt ") {
            self.elements
                .push(Element::AltStart(line[4..].trim().to_string()));
            return;
        }
        if line == "else" || line.starts_with("else ") {
            let cond = if line.len() > 4 {
                Some(line[4..].trim().to_string())
            } else {
                None
            };
            self.elements.push(Element::ElseBranch(cond));
            return;
        }
        if line == "end" {
            self.elements.push(Element::AltEnd);
            return;
        }

        // Message: A -> B: text
        self.try_parse_message(line);
    }

    fn parse_participant(&mut self, rest: &str) {
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let name = parts[0].to_string();
        let mut order = self.participant_order;

        // Check for "order N"
        if let Some(pos) = parts.iter().position(|&s| s == "order") {
            if let Some(n) = parts.get(pos + 1) {
                if let Ok(o) = n.parse() {
                    order = o;
                }
            }
        }

        if !self.participants.contains_key(&name) {
            self.participants.insert(
                name.clone(),
                Participant {
                    name,
                    order,
                    x: 0.0,
                    width: 0.0,
                },
            );
            self.participant_order += 1;
        }
    }

    fn try_parse_message(&mut self, line: &str) {
        // Arrow patterns: ->>, -->, ->, -->>
        let patterns = [
            ("-->>", ArrowStyle::DashedOpen),
            ("->>", ArrowStyle::SolidOpen),
            ("-->", ArrowStyle::Dashed),
            ("->", ArrowStyle::Solid),
        ];

        for (pattern, style) in patterns {
            if let Some(pos) = line.find(pattern) {
                let from = line[..pos].trim();
                let rest = &line[pos + pattern.len()..];

                // Split on colon for message text
                let (to, text) = if let Some(colon) = rest.find(':') {
                    (rest[..colon].trim(), rest[colon + 1..].trim())
                } else {
                    (rest.trim(), "")
                };

                if !from.is_empty() && !to.is_empty() {
                    // Ensure participants exist
                    self.ensure_participant(from);
                    self.ensure_participant(to);

                    self.elements.push(Element::Message(Message {
                        from: from.to_string(),
                        to: to.to_string(),
                        text: text.to_string(),
                        style,
                    }));
                }
                return;
            }
        }
    }

    fn ensure_participant(&mut self, name: &str) {
        if !self.participants.contains_key(name) {
            self.participants.insert(
                name.to_string(),
                Participant {
                    name: name.to_string(),
                    order: self.participant_order,
                    x: 0.0,
                    width: 0.0,
                },
            );
            self.participant_order += 1;
        }
    }
}

// ============================================================================
// Layout
// ============================================================================

impl SequenceDiagram {
    fn layout(&mut self, style: &DiagramStyle) {
        let _participant_height = 35.0;
        let participant_padding = 20.0;
        let participant_spacing = 150.0;

        // Calculate participant widths
        for p in &mut self.participants {
            p.width = p.name.len() as f32 * style.char_width + participant_padding * 2.0;
            p.width = p.width.max(80.0);
        }

        // Position participants
        let mut current_x = style.margin;
        for p in &mut self.participants {
            p.x = current_x + p.width / 2.0;
            current_x += p.width.max(participant_spacing);
        }
    }

    fn calculate_dimensions(&self, style: &DiagramStyle) -> (f32, f32) {
        let participant_height = 35.0;
        let message_spacing = 40.0;

        // Width
        let width = if let Some(last) = self.participants.last() {
            last.x + last.width / 2.0 + style.margin
        } else {
            200.0
        };

        // Height: count elements
        let mut element_count = 0;
        let mut alt_depth: usize = 0;

        for elem in &self.elements {
            match elem {
                Element::Message(_) | Element::Divider(_) => element_count += 1,
                Element::AltStart(_) => {
                    element_count += 1;
                    alt_depth += 1;
                }
                Element::ElseBranch(_) => element_count += 1,
                Element::AltEnd => {
                    element_count += 1;
                    alt_depth = alt_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        let height = style.margin * 2.0
            + participant_height * 2.0
            + element_count as f32 * message_spacing
            + 40.0;

        (width, height)
    }
}

// ============================================================================
// Renderer
// ============================================================================

pub fn render(source: &str, style: &DiagramStyle) -> String {
    let mut diagram = Parser::new().parse(source);
    diagram.layout(style);

    let (width, height) = diagram.calculate_dimensions(style);
    let custom_css = crate::common::extract_custom_css(source);
    let mut svg = SvgBuilder::new(width, height, style, custom_css.as_deref());

    // Arrow markers with CSS classes
    svg.push(
        r#"<defs>
<marker id="seq-arrow" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
<polygon points="0 0, 10 3.5, 0 7" class="arrow-head"/>
</marker>
<marker id="seq-arrow-open" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
<polyline points="0 0, 10 3.5, 0 7" class="arrow-head-open"/>
</marker>
</defs>"#,
    );

    let participant_height = 35.0;
    let top_y = style.margin;
    let bottom_y = height - style.margin - participant_height;

    // Draw lifelines
    for p in &diagram.participants {
        svg.line_class(p.x, top_y + participant_height, p.x, bottom_y, "lifeline");
    }

    // Draw participant boxes (top and bottom)
    for p in &diagram.participants {
        draw_participant_box(&mut svg, p, top_y, participant_height, style);
        draw_participant_box(&mut svg, p, bottom_y, participant_height, style);
    }

    // Draw elements
    let mut current_y = top_y + participant_height + 30.0;
    let message_spacing = 40.0;
    let mut alt_stack: Vec<(f32, f32, f32)> = Vec::new(); // (start_y, left_x, right_x)

    for elem in &diagram.elements {
        match elem {
            Element::Message(msg) => {
                draw_message(&mut svg, &diagram.participants, msg, current_y, style);
                current_y += message_spacing;
            }
            Element::Divider(text) => {
                draw_divider(&mut svg, width, current_y, text, style);
                current_y += message_spacing;
            }
            Element::AltStart(cond) => {
                let (left_x, right_x) = get_diagram_bounds(&diagram.participants, style);
                alt_stack.push((current_y, left_x, right_x));

                // Draw alt header
                svg.text_class(
                    left_x + 5.0,
                    current_y + 15.0,
                    &format!("[{}]", cond),
                    "alt-condition-text",
                );
                current_y += message_spacing;
            }
            Element::ElseBranch(cond) => {
                if let Some(&(_, left_x, right_x)) = alt_stack.last() {
                    // Dashed line for else
                    svg.line_class(left_x, current_y, right_x, current_y, "alt-divider");

                    if let Some(c) = cond {
                        svg.text_class(
                            left_x + 5.0,
                            current_y + 15.0,
                            &format!("[{}]", c),
                            "alt-condition-text diagram-text",
                        );
                    }
                }
                current_y += message_spacing * 0.5;
            }
            Element::AltEnd => {
                if let Some((start_y, left_x, right_x)) = alt_stack.pop() {
                    // Draw alt box
                    let box_height = current_y - start_y;
                    svg.push(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" class="alt-box"/>"#,
                        left_x,
                        start_y,
                        right_x - left_x,
                        box_height
                    ));
                    // Alt label box
                    svg.polygon_class(
                        &[
                            (left_x, start_y),
                            (left_x + 30.0, start_y),
                            (left_x + 40.0, start_y + 15.0),
                            (left_x, start_y + 15.0),
                        ],
                        "alt-label-box",
                    );
                    svg.text_class(left_x + 5.0, start_y + 11.0, "alt", "alt-label-text");
                }
                current_y += message_spacing * 0.5;
            }
            _ => {}
        }
    }

    svg.finish()
}

fn draw_participant_box(
    svg: &mut SvgBuilder,
    p: &Participant,
    y: f32,
    height: f32,
    _style: &DiagramStyle,
) {
    let x = p.x - p.width / 2.0;
    svg.rect_class(x, y, p.width, height, "participant");
    svg.text_class(p.x, y + height / 2.0 + 4.0, &p.name, "participant-text");
}

fn draw_message(
    svg: &mut SvgBuilder,
    participants: &[Participant],
    msg: &Message,
    y: f32,
    _style: &DiagramStyle,
) {
    let from_p = participants.iter().find(|p| p.name == msg.from);
    let to_p = participants.iter().find(|p| p.name == msg.to);

    let (from_p, to_p) = match (from_p, to_p) {
        (Some(f), Some(t)) => (f, t),
        _ => return,
    };

    let dashed = matches!(msg.style, ArrowStyle::Dashed | ArrowStyle::DashedOpen);
    let marker = match msg.style {
        ArrowStyle::Solid | ArrowStyle::Dashed => "url(#seq-arrow)",
        ArrowStyle::SolidOpen | ArrowStyle::DashedOpen => "url(#seq-arrow-open)",
    };

    let class = if dashed {
        "message message-dashed"
    } else {
        "message"
    };

    if msg.from == msg.to {
        // Self-message
        let loop_width = 30.0;
        let loop_height = 20.0;
        let points = vec![
            (from_p.x, y),
            (from_p.x + loop_width, y),
            (from_p.x + loop_width, y + loop_height),
            (from_p.x, y + loop_height),
        ];
        svg.polyline_class(&points, class, marker);

        svg.text_class(
            from_p.x + loop_width + 5.0,
            y + loop_height / 2.0 + 4.0,
            &msg.text,
            "message-text",
        );
    } else {
        // Normal message
        let (x1, x2) = (from_p.x, to_p.x);
        svg.polyline_class(&[(x1, y), (x2, y)], class, marker);

        // Label
        let mid_x = (x1 + x2) / 2.0;
        svg.text_class(mid_x, y - 5.0, &msg.text, "message-text");
    }
}

fn draw_divider(svg: &mut SvgBuilder, width: f32, y: f32, text: &str, style: &DiagramStyle) {
    let left = style.margin;
    let right = width - style.margin;

    // Dashed line
    svg.line_class(left, y, right, y, "divider-line");

    // Text box in center
    let text_width = text.len() as f32 * style.char_width + 20.0;
    let box_x = (width - text_width) / 2.0;

    svg.rect_class(box_x, y - 10.0, text_width, 20.0, "divider-box");
    svg.text_class(width / 2.0, y + 4.0, text, "divider-text");
}

fn get_diagram_bounds(participants: &[Participant], style: &DiagramStyle) -> (f32, f32) {
    let left = participants
        .first()
        .map(|p| p.x - p.width / 2.0 - 10.0)
        .unwrap_or(style.margin);
    let right = participants
        .last()
        .map(|p| p.x + p.width / 2.0 + 10.0)
        .unwrap_or(200.0);
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let source = "@start_uml\nparticipant A\nA -> B: hello\n@end_uml";
        let diagram = Parser::new().parse(source);
        assert_eq!(diagram.participants.len(), 2);
        assert_eq!(diagram.elements.len(), 1);
    }

    #[test]
    fn test_self_message() {
        let source = "@start_uml\nA -> A: self\n@end_uml";
        let diagram = Parser::new().parse(source);
        if let Element::Message(msg) = &diagram.elements[0] {
            assert_eq!(msg.from, msg.to);
        }
    }
}
