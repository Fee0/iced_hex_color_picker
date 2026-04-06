use crate::style::{GridDrawStyle, MapColorTarget};
use hex_color::Rgb;
use iced::mouse;
use iced::widget::canvas;
use iced::widget::text as w_text;
use iced::{alignment, Color, Point, Rectangle, Renderer, Size, Theme};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum GridMessage {
    SelectionChanged { start: u8, end: u8 },
    DragEnded,
}

pub struct GridProgram<'a> {
    pub colors: &'a [Rgb; 256],
    pub selection: Option<(u8, u8)>,
    pub draw_style: GridDrawStyle,
}

#[derive(Default)]
pub struct GridInteraction {
    anchor: Option<u8>,
    dragging: bool,
}

fn label_contrast_for_rgb(rgb: Rgb) -> Color {
    let lum = 0.299 * rgb.r as f32 + 0.587 * rgb.g as f32 + 0.114 * rgb.b as f32;
    if lum > 128.0 {
        Color::BLACK
    } else {
        Color::WHITE
    }
}

fn hit_test(pos: Point, size: Size) -> Option<u8> {
    let cw = size.width / 16.0;
    let ch = size.height / 16.0;
    let col = (pos.x / cw) as usize;
    let row = (pos.y / ch) as usize;
    if col < 16 && row < 16 {
        Some((row * 16 + col) as u8)
    } else {
        None
    }
}

/// Corner of the logical 17×17 grid (cell (c,r) uses corners (c,r)…(c+1,r+1)).
type GridCorner = (u8, u8);

fn norm_edge(a: GridCorner, b: GridCorner) -> (GridCorner, GridCorner) {
    if a.0 < b.0 || (a.0 == b.0 && a.1 < b.1) {
        (a, b)
    } else {
        (b, a)
    }
}

fn xor_cell_edges(boundary: &mut HashSet<(GridCorner, GridCorner)>, col: u8, row: u8) {
    let c = col;
    let r = row;
    let edges = [
        norm_edge((c, r), (c + 1, r)),
        norm_edge((c + 1, r), (c + 1, r + 1)),
        norm_edge((c + 1, r + 1), (c, r + 1)),
        norm_edge((c, r + 1), (c, r)),
    ];
    for e in edges {
        if !boundary.remove(&e) {
            boundary.insert(e);
        }
    }
}

fn selection_boundary_edges(lo: u8, hi: u8) -> HashSet<(GridCorner, GridCorner)> {
    let mut boundary = HashSet::new();
    for b in lo..=hi {
        xor_cell_edges(&mut boundary, b % 16, b / 16);
    }
    boundary
}

fn corner_to_point(c: GridCorner, cw: f32, ch: f32) -> Point {
    Point::new(c.0 as f32 * cw, c.1 as f32 * ch)
}

/// Traces closed loops from the XOR boundary edge set; consumes `edge_rem`.
fn boundary_loops_as_paths(edge_rem: &mut HashSet<(GridCorner, GridCorner)>, cw: f32, ch: f32) -> Vec<canvas::Path> {
    let mut adj: HashMap<GridCorner, Vec<GridCorner>> = HashMap::new();
    for &(a, b) in edge_rem.iter() {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    let mut paths = Vec::new();

    while let Some(&(c0, c1)) = edge_rem.iter().next() {
        let mut poly = vec![c0];
        let mut prev = c0;
        let mut cur = c1;
        // Only follow edges still in `edge_rem`; static `adj` includes removed edges, which
        // otherwise allowed picking a "ghost" neighbor and spinning forever without reaching c0.
        let max_steps = edge_rem.len() + 64;
        let mut steps = 0usize;

        loop {
            if steps >= max_steps {
                edge_rem.clear();
                return paths;
            }
            steps += 1;

            let e = norm_edge(prev, cur);
            if !edge_rem.remove(&e) {
                edge_rem.clear();
                return paths;
            }
            poly.push(cur);
            if cur == c0 {
                break;
            }
            let neighbors = adj.get(&cur).cloned().unwrap_or_default();
            let Some(next) = neighbors.into_iter().find(|&n| {
                n != prev && edge_rem.contains(&norm_edge(cur, n))
            }) else {
                edge_rem.clear();
                return paths;
            };
            prev = cur;
            cur = next;
        }

        let path = canvas::Path::new(|b| {
            let p0 = corner_to_point(poly[0], cw, ch);
            b.move_to(p0);
            for c in poly.iter().skip(1) {
                b.line_to(corner_to_point(*c, cw, ch));
            }
            b.close();
        });
        paths.push(path);
    }

    paths
}

impl<'a> canvas::Program<GridMessage> for GridProgram<'a> {
    type State = GridInteraction;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<GridMessage>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some(b) = hit_test(pos, bounds.size()) {
                        state.anchor = Some(b);
                        state.dragging = true;
                        return Some(
                            canvas::Action::publish(GridMessage::SelectionChanged {
                                start: b,
                                end: b,
                            })
                            .and_capture(),
                        );
                    }
                }
                None
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                if let Some(pos) = cursor.position_in(bounds) {
                    if let Some(b) = hit_test(pos, bounds.size()) {
                        if let Some(a) = state.anchor {
                            return Some(
                                canvas::Action::publish(GridMessage::SelectionChanged {
                                    start: a.min(b),
                                    end: a.max(b),
                                })
                                .and_capture(),
                            );
                        }
                    }
                }
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging =>
            {
                state.dragging = false;
                Some(canvas::Action::publish(GridMessage::DragEnded).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let sz = bounds.size();
        let mut frame = canvas::Frame::new(renderer, sz);
        let cw = sz.width / 16.0;
        let ch = sz.height / 16.0;
        let st = self.draw_style;

        for i in 0u16..256 {
            let b = i as u8;
            let x = (i % 16) as f32 * cw;
            let y = (i / 16) as f32 * ch;
            let rgb = self.colors[b as usize];
            let fill = match st.map_color_target {
                MapColorTarget::Text => st.cell_background,
                MapColorTarget::CellFill => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
            };
            frame.fill_rectangle(Point::new(x, y), Size::new(cw, ch), fill);
        }

        if let Some((lo, hi)) = self.selection {
            for b in lo..=hi {
                let i = b as u16;
                let x = (i % 16) as f32 * cw;
                let y = (i / 16) as f32 * ch;
                frame.fill_rectangle(
                    Point::new(x, y),
                    Size::new(cw, ch),
                    st.selection_overlay,
                );
            }

            let mut edges = selection_boundary_edges(lo, hi);
            let stroke = canvas::Stroke::default()
                .with_color(st.selection_outline)
                .with_width(st.selection_outline_width);
            for path in boundary_loops_as_paths(&mut edges, cw, ch) {
                frame.stroke(&path, stroke);
            }
        }

        for i in 0u16..256 {
            let b = i as u8;
            let x = (i % 16) as f32 * cw;
            let y = (i / 16) as f32 * ch;
            let rgb = self.colors[b as usize];
            let label_color = match st.map_color_target {
                MapColorTarget::Text => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
                MapColorTarget::CellFill => label_contrast_for_rgb(rgb),
            };
            frame.fill_text(canvas::Text {
                content: format!("{b:02X}"),
                position: Point::new(x + cw / 2.0, y + ch / 2.0),
                color: label_color,
                size: 11.0.into(),
                font: iced::Font::MONOSPACE,
                align_x: w_text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
        }

        for j in 0..=16 {
            let x = j as f32 * cw;
            let y = j as f32 * ch;
            frame.stroke(
                &canvas::Path::line(Point::new(x, 0.0), Point::new(x, sz.height)),
                canvas::Stroke::default()
                    .with_color(st.grid_line_color)
                    .with_width(0.5),
            );
            frame.stroke(
                &canvas::Path::line(Point::new(0.0, y), Point::new(sz.width, y)),
                canvas::Stroke::default()
                    .with_color(st.grid_line_color)
                    .with_width(0.5),
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Crosshair
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
