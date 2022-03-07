use chrono::{Duration, Local, Timelike};
use std::f32::consts::TAU;
use std::{thread, time};
// use egui::{containers::*, widgets::*, *};
use eframe::egui::{self, CollapsingHeader, Context, Frame, Slider};
use eframe::egui::{
    emath, emath::pos2, CentralPanel, Color32, Painter, Pos2, Rect, Shape, Ui, Vec2,
};
use eframe::epaint::{RectShape, Stroke};
use eframe::epi::{self, Storage};
use egui::{color::*, widgets::color_picker::show_color, *};
use std::sync::mpsc::{channel, Sender};

const BLACK: Color32 = Color32::BLACK;
const WHITE: Color32 = Color32::WHITE;
const GREEN: Color32 = Color32::GREEN;
const RED: Color32 = Color32::RED;
const TRANSPARENT: Color32 = Color32::TRANSPARENT;

#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct H24Clock {
    radius: f32,
    border_width: f32,
    hour_hand_color: Color32,
    minute_hand_color: Color32,
    second_hand_color: Color32,
    number_color: Color32,
    border_color: Color32,
    ruling_distance: f32,
    ruling_pos: Vec<Pos2>,
    txt_distance: f32,
    txt_pos: Vec<Pos2>,

    paused: bool,
    time: f64,
    zoom: f32,
    start_line_width: f32,
    depth: usize,
    length_factor: f32,
    luminance_factor: f32,
    width_factor: f32,
    line_count: usize,
}
pub struct ClockApp {
    h24: H24Clock,
    sender: Option<Sender<Context>>,
}
impl Default for H24Clock {
    fn default() -> Self {
        let r0: f32 = 2.0;
        let scale = r0 / 250.0;
        let border = r0 / 50.0;
        let ruling_dist = r0 - border * 3.0;
        let txt_dist = r0 - border * 5.5;
        let get_pos = |v: f32, distance| pos2(v.cos() * distance, v.sin() * distance);
        let mut points: Vec<Pos2> = vec![];
        let mut ptxt: Vec<Pos2> = vec![];
        //init seconds offset
        for i in 0..120 {
            let v = (i as f32 * 3.0 - 90.0).to_radians();
            let seconds_xy = get_pos(v, ruling_dist);
            if i % 5 == 0 {
                let txt_xy = get_pos(v, txt_dist);
                ptxt.push(txt_xy);
            }
            points.push(seconds_xy);
        }
        Self {
            radius: r0,
            border_width: border,
            hour_hand_color: WHITE,
            minute_hand_color: WHITE,
            second_hand_color: WHITE,
            number_color: WHITE,
            border_color: WHITE,
            ruling_distance: ruling_dist,
            ruling_pos: points,
            txt_distance: txt_dist,
            txt_pos: ptxt,

            paused: false,
            time: 0.0,
            zoom: 0.25,
            start_line_width: 2.5,
            depth: 9,
            length_factor: 0.8,
            luminance_factor: 0.8,
            width_factor: 0.9,
            line_count: 0,
        }
    }
}
impl Default for ClockApp {
    fn default() -> Self {
        Self {
            h24: H24Clock::default(),
            sender: None,
        }
    }
}

impl epi::App for ClockApp {
    fn name(&self) -> &str {
        "🕑 24 Hours Clock"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default()
            .frame(Frame::dark_canvas(&ctx.style()))
            .show(ctx, |ui| self.ui(ui, seconds_since_midnight()));
    }
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &eframe::epi::Frame,
        _storage: Option<&dyn Storage>,
    ) {
        let (send, recv) = channel::<Context>();
        self.sender = Some(send);
        println!("setup called");
    }
}

impl ClockApp {
    pub fn ui(&mut self, ui: &mut Ui, seconds_since_midnight: Option<f64>) {
        if !self.h24.paused {
            self.h24.time = seconds_since_midnight.unwrap_or_else(|| ui.input().time);
            ui.ctx().request_repaint();
        }

        let painter = Painter::new(
            ui.ctx().clone(),
            ui.layer_id(),
            ui.available_rect_before_wrap(),
        );
        self.paint(&painter);
        // Make sure we allocate what we used (everything)
        ui.expand_to_include_rect(painter.clip_rect());

        Frame::popup(ui.style())
            .stroke(Stroke::none())
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                CollapsingHeader::new("Settings")
                    .show(ui, |ui| self.options_ui(ui, seconds_since_midnight));
            });
    }

    fn options_ui(&mut self, ui: &mut Ui, seconds_since_midnight: Option<f64>) {
        if seconds_since_midnight.is_some() {
            ui.label(format!(
                "Local time: {:02}:{:02}:{:02}.{:03}",
                (self.h24.time % (24.0 * 60.0 * 60.0) / 3600.0).floor(),
                (self.h24.time % (60.0 * 60.0) / 60.0).floor(),
                (self.h24.time % 60.0).floor(),
                (self.h24.time % 1.0 * 100.0).floor()
            ));
        } else {
            ui.label("The fractal_clock clock is not showing the correct time");
        };
        ui.label(format!("Painted line count: {}", self.h24.line_count));

        ui.checkbox(&mut self.h24.paused, "Paused");
        ui.add(Slider::new(&mut self.h24.zoom, 0.0..=1.0).text("zoom"));
        ui.add(Slider::new(&mut self.h24.start_line_width, 0.0..=5.0).text("Start line width"));
        ui.add(Slider::new(&mut self.h24.depth, 0..=14).text("depth"));
        ui.add(Slider::new(&mut self.h24.length_factor, 0.0..=1.0).text("length factor"));
        ui.add(Slider::new(&mut self.h24.luminance_factor, 0.0..=1.0).text("luminance factor"));
        ui.add(Slider::new(&mut self.h24.width_factor, 0.0..=1.0).text("width factor"));

        egui::reset_button(ui, &mut self.h24);

        // ui.hyperlink_to(
        //     "Inspired by a screensaver by Rob Mayoff",
        //     "http://www.dqd.com/~mayoff/programs/ClockApp/",
        // );
        // ui.add(crate::__egui_github_link_file!());
    }

    fn paint(&mut self, painter: &Painter) {
        struct Hand {
            length: f32,
            angle: f32,
            vec: Vec2,
        }

        impl Hand {
            fn from_length_angle(length: f32, angle: f32) -> Self {
                Self {
                    length,
                    angle,
                    vec: length * Vec2::angled(angle),
                }
            }
        }

        let angle_from_period =
            |period| TAU * (self.h24.time.rem_euclid(period) / period) as f32 + TAU / 4.0;

        let hands = [
            // Second hand:
            Hand::from_length_angle(self.h24.length_factor, angle_from_period(120.0)),
            // Minute hand:
            Hand::from_length_angle(self.h24.length_factor, angle_from_period(120.0 * 60.0)),
            // Hour hand:
            Hand::from_length_angle(0.5, angle_from_period(24.0 * 60.0 * 60.0)),
        ];

        let mut shapes: Vec<Shape> = Vec::new();

        let rect = painter.clip_rect();
        let to_screen = emath::RectTransform::from_to(
            Rect::from_center_size(Pos2::ZERO, rect.square_proportions() / self.h24.zoom),
            rect,
        );
        let mut paint_line = |points: [Pos2; 2], color: Color32, width: f32| {
            let line = [to_screen * points[0], to_screen * points[1]];
            // println!("to_screen {:?}", to_screen);
            // println!("pointer0 {:?}", points[0]);
            // println!("line {:?}", line);
            // println!("rect {:?}", rect);
            // culling
            if rect.intersects(Rect::from_two_pos(line[0], line[1])) {
                shapes.push(Shape::line_segment(line, (width, color)));
            }
        };

        let hand_rotations = [
            hands[0].angle - hands[2].angle + TAU / 2.0,
            hands[1].angle - hands[2].angle + TAU / 2.0,
        ];

        let hand_rotors = [
            hands[0].length * emath::Rot2::from_angle(hand_rotations[0]),
            hands[1].length * emath::Rot2::from_angle(hand_rotations[1]),
        ];

        #[derive(Clone, Copy)]
        struct Node {
            pos: Pos2,
            dir: Vec2,
        }

        let mut nodes = Vec::new();

        let mut width = self.h24.start_line_width;

        for (i, hand) in hands.iter().enumerate() {
            // extand hand.
	    let color = match i {
		0 => RED,
		1 => GREEN,
		_ => Color32::BLUE,
	    };
            let luminance_u8 = (255.0 * 0.8f32).round() as u8;
            let start = (-0.5 * hand.vec).to_pos2();
            let end = (2.1 * hand.vec).to_pos2();
            paint_line(
                [start, end],
                // Color32::from_additive_luminance(luminance_u8),
		color,
                width * 0.93,
            );
            let center = pos2(0.0, 0.0);
            let end = center + hand.vec;
            paint_line([center, end], Color32::from_additive_luminance(255), width);
            if i < 2 {
                nodes.push(Node {
                    pos: end,
                    dir: hand.vec,
                });
            }
        }

        let mut luminance = 0.7; // Start dimmer than main hands

        let mut new_nodes = Vec::new();
        for _ in 0..self.h24.depth {
            new_nodes.clear();
            new_nodes.reserve(nodes.len() * 2);

            luminance *= self.h24.luminance_factor;
            width *= self.h24.width_factor;

            let luminance_u8 = (255.0 * luminance).round() as u8;
            if luminance_u8 == 0 {
                break;
            }

            for &rotor in &hand_rotors {
                for a in &nodes {
                    let new_dir = rotor * a.dir;
                    let b = Node {
                        pos: a.pos + new_dir,
                        dir: new_dir,
                    };
                    paint_line(
                        [a.pos, b.pos],
                        Color32::from_additive_luminance(luminance_u8),
                        width,
                    );
                    new_nodes.push(b);
                }
            }

            std::mem::swap(&mut nodes, &mut new_nodes);
        }
        // draw board.
        self.h24.line_count = shapes.len();
        let r0 = self.h24.radius;
        let center = to_screen * pos2(0.0, 0.0);
        // shapes.push(Shape::circle_stroke(
        //     center,
        //     r0,
        //     Stroke::new(self.h24.border_width, self.h24.border_color),
        // ));
        let pos0 = to_screen * pos2(0.0, r0);
        let bw = to_screen * pos2(0.0, self.h24.border_width);
        shapes.push(Shape::circle_stroke(
            center,
            pos0.y - center.y - 4.0,
            Stroke::new(bw.y - center.y, self.h24.border_color),
        ));
        // draw yellow circle
        shapes.push(Shape::circle_stroke(
            center,
            3.0,
            Stroke::new(2.0, Color32::YELLOW),
        ));

        // draw ruling.
        for (i, p) in self.h24.ruling_pos.iter().enumerate() {
            // let r0 = self.h24.ruling_distance;
            // let transfer = center - pos2(r0, r0);
            // let p0 = *p + transfer;
            let p0 = to_screen * (*p);
            // let weight;
            // if i % 5 == 0 { weight = 4.0} else {weight = 2.0}
            let weight = if i % 5 == 0 { 4.0 } else { 2.0 };
            let delta = Vec2::new(5.0, 5.0);
            shapes.push(Shape::rect_filled(
                Rect::from_two_pos(p0, p0 + Vec2::new(weight, weight)),
                Rounding::default(),
                WHITE,
            ));
            // draw number text.
            if i % 5 == 0 {
                let index = i / 5;
                // let r0 = self.h24.txt_distance;
                // let transfer = center - pos2(r0, r0);
                // let p0 = self.h24.txt_pos[index] + transfer;
                let p0 = to_screen * self.h24.txt_pos[index];
                let k = (index % 12) as u8;
                let h12 = if k == 0 { 12 } else { k };
                let ctx = painter.ctx();
                let style = ctx.style().clone();
                let fonts = ctx.fonts();
                shapes.push(Shape::text(
                    &fonts,
                    p0,
                    Align2::CENTER_CENTER,
                    h12.to_string(),
                    egui::TextStyle::Monospace.resolve(style.as_ref()),
                    WHITE,
                ));
            }
        }
        painter.extend(shapes);
    }
}

fn line_hand(ang: f32, scale: (f32, f32)) -> (Vec2, Vec2) {
    let line_scale = |ang: f32, scale| Vec2 {
        x: ang.cos() * scale,
        y: ang.sin() * scale,
    };
    (line_scale(ang, scale.0), -line_scale(ang, scale.1))
}
/// Time of day as seconds since midnight
fn seconds_since_midnight() -> Option<f64> {
    let time = chrono::Local::now().time();
    let seconds_since_midnight =
        time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64);
    Some(seconds_since_midnight)
}
