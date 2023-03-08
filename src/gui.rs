use iced::Application;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord {
    x: f32,
    y: f32,
}

#[derive(Debug, Default)]
struct GraphicsHolder {
    dots: Vec<Coord>,
    lines: Vec<(Coord, Coord)>,
}

#[derive(Debug, Default)]
pub struct DrawingApi {
    holder: GraphicsHolder,
}

#[derive(Debug, Clone, Copy)]
pub enum DrawBackend {
    Egui,
    Iced,
}

impl clap::ValueEnum for DrawBackend {
    fn value_variants<'a>() -> &'a [Self] {
        &[DrawBackend::Egui, DrawBackend::Iced]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(match self {
            DrawBackend::Egui => "egui",
            DrawBackend::Iced => "iced",
        }))
    }
}

impl DrawingApi {
    pub fn draw_dot(&mut self) -> Coord {
        let mut gen = rand::thread_rng();
        let x = gen.gen_range(0.0..1.0);
        let y = gen.gen_range(0.0..1.0);
        let coord = Coord { x, y };

        self.holder.dots.push(coord);
        coord
    }

    pub fn draw_edge(&mut self, from: Coord, to: Coord) {
        self.holder.lines.push((from, to));
    }

    pub fn draw_with(self, backend_type: DrawBackend) {
        match backend_type {
            DrawBackend::Egui => eframe::run_native(
                "Graph draw egui",
                eframe::NativeOptions::default(),
                Box::new(|_| Box::new(egui_backend::DrawBackend::new(self.holder))),
            )
            .unwrap_or_else(|err| eprintln!("Egui backend failed with {err}")),
            DrawBackend::Iced => {
                iced_backend::DrawBackend::run(iced::Settings::with_flags(self.holder))
                    .unwrap_or_else(|err| eprintln!("Iced backend failed with {err}"))
            }
        }
    }
}

pub mod iced_backend {
    impl Mul<iced::Size> for Coord {
        type Output = iced::Point;

        fn mul(self, size: iced::Size) -> Self::Output {
            iced::Point {
                x: self.x * size.width,
                y: self.y * size.height,
            }
        }
    }

    #[derive(Debug)]
    pub(super) struct DrawBackend {
        canvas_drawer: CanvasDrawer,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub(super) struct Message;

    use std::{fmt::Debug, ops::Mul};

    use iced::{widget::canvas, Application};

    use super::{Coord, GraphicsHolder};

    impl Application for DrawBackend {
        type Message = Message;
        type Executor = iced::executor::Default;
        type Theme = iced::Theme;
        type Flags = GraphicsHolder;

        fn new(graph: Self::Flags) -> (Self, iced::Command<Self::Message>) {
            (
                Self {
                    canvas_drawer: CanvasDrawer {
                        holder: graph,
                        ..Default::default()
                    },
                },
                iced::Command::none(),
            )
        }

        fn title(&self) -> String {
            "iced-based graphs".to_string()
        }

        fn update(&mut self, _message: Self::Message) -> iced::Command<Message> {
            iced::Command::none()
        }

        fn view(&self) -> iced::Element<'_, Self::Message> {
            iced::widget::column!(iced::widget::canvas(&self.canvas_drawer)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill))
            .width(iced::Length::Fill)
            .align_items(iced::Alignment::Center)
            .into()
        }
    }

    #[derive(Debug, Default)]
    struct CanvasDrawer {
        cache: canvas::Cache,
        holder: GraphicsHolder,
    }

    impl canvas::Program<Message> for CanvasDrawer {
        type State = GraphicsHolder;

        fn draw(
            &self,
            _state: &Self::State,
            _theme: &iced::Theme,
            bounds: iced::Rectangle,
            _cursor: canvas::Cursor,
        ) -> Vec<canvas::Geometry> {
            let geom = self.cache.draw(bounds.size(), |frame| {
                frame.fill(
                    &canvas::Path::rectangle(iced::Point::new(0.0, 0.0), frame.size()),
                    iced::Color::from_rgb8(0x20, 0x20, 0x20),
                );

                let white = iced::Color::from_rgb8(0xff, 0xff, 0xff);
                for dot in self.holder.dots.iter().copied() {
                    let dot_form = canvas::Path::circle(dot * frame.size(), 5.0);
                    frame.fill(&dot_form, white)
                }

                for (from, to) in self.holder.lines.iter().copied() {
                    let line = canvas::Path::line(from * frame.size(), to * frame.size());
                    frame.stroke(&line, canvas::Stroke::default().with_color(white))
                }
            });

            vec![geom]
        }
    }
}

mod egui_backend {
    use std::ops::Mul;

    use super::{Coord, GraphicsHolder};

    impl Mul<egui::Vec2> for Coord {
        type Output = egui::Pos2;

        fn mul(self, size: egui::Vec2) -> Self::Output {
            egui::Pos2 {
                x: self.x * size.x,
                y: self.y * size.y,
            }
        }
    }

    #[derive(Debug, Default)]
    pub(super) struct DrawBackend {
        graph: GraphicsHolder,
    }

    impl DrawBackend {
        pub(super) fn new(graph: GraphicsHolder) -> Self {
            DrawBackend { graph }
        }

        fn draw_once(&self, ctx: &egui::Context) {
            egui::CentralPanel::default().show(ctx, |ui| {
                let painter = ui.painter();
                let white = egui::Color32::from_rgb(0xff, 0xff, 0xff);

                for dot in self.graph.dots.iter().copied() {
                    painter.circle_filled(dot * ui.available_size(), 5.0, white);
                }

                for (from, to) in self.graph.lines.iter().copied() {
                    let from = from * ui.available_size();
                    let to = to * ui.available_size();
                    painter.line_segment([from, to], (1.0, white));
                }
            });
        }
    }

    impl eframe::App for DrawBackend {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.draw_once(ctx)
        }
    }
}
