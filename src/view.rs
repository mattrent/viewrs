use core::f32;
use std::path::PathBuf;

use crate::export::{export_png, generate_png, generate_svg_tree};
use crate::ext_svg::ExtendedSvg;
use crate::layers::set_visible_layers;
use crate::transform::transform_svg;
use iced::mouse::{
    Button::Left,
    Event::{ButtonPressed, ButtonReleased, CursorMoved, WheelScrolled},
    ScrollDelta,
};
use iced::widget::{
    button, checkbox, column, container, row, svg, text, text_input, Checkbox, Container, Row,
};
use iced::Event::{Mouse, Window};
use iced::{
    event, executor, font, theme, Application, Command, Element, Length, Point, Subscription,
};
use iced_aw::number_input;
use iced_aw::widgets::Modal;
use iced_style::core::window;
use resvg::usvg::{Size, Tree};

#[derive(Debug, Default)]
pub(crate) struct Picture {
    // graphical properties
    ask_overwrite: bool,
    show_modal: bool,
    panning: bool,
    current_scroll: f32,
    current_x: f32,
    current_y: f32,
    current_width: u32,
    current_height: u32,

    // content + layers
    file_name: String,
    svg_content: Vec<u8>,
    layers: Vec<(String, bool)>,
    matrix_transform: (f32, f32, f32, f32, f32, f32),

    // PNG + structure
    png_content: Vec<u8>,
    svg_tree: Option<Tree>,
    height: f32,
    width: f32,
    ratio: f32,

    // output properties
    output_file_name: String,
    output_width: f32,
    output_height: f32,
}

#[derive(Debug, Default)]
pub(crate) struct PictureFlags {
    pub(crate) file_name: String,
    pub(crate) svg_content: Vec<u8>,
    pub(crate) layers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // layers
    ToggleLayerVisibility(String, bool),
    // output information
    OutFileName(String),
    OutWidth(f32),
    OutHeight(f32),
    // export + overwrite modals
    OpenExport,
    SaveExport,
    CancelExport,
    Overwrite,
    NoOverwrite,
    // preload
    FontLoaded,
    // events
    Scroll(f32),
    StartPan,
    CursorMoved(f32, f32),
    EndPan,
    Reset,
    Resized(u32, u32),
}

impl Picture {
    fn save_modal(&self) -> Container<Message> {
        container(
            column![column![
                text("Path").size(16),
                text_input("", &self.output_file_name)
                    .on_input(Message::OutFileName)
                    .width(Length::Fixed(500.0))
                    .size(16),
                text("Width").size(16),
                number_input(self.output_width, f32::MAX, Message::OutWidth).size(16.0),
                text("Height").size(16),
                number_input(self.output_height, f32::MAX, Message::OutHeight).size(16.0),
                container(
                    row![
                        button(text("Cancel").size(16)).on_press(Message::CancelExport),
                        button(text("Save").size(16)).on_press(Message::SaveExport),
                    ]
                    .spacing(20)
                )
                .center_x()
            ]
            .spacing(10)]
            .spacing(20),
        )
        .padding(15)
        .style(theme::Container::Box)
    }

    fn overwrite_modal(&self) -> Container<Message> {
        container(
            column![column![
                column![text("File exists. Overwrite?").size(16),].spacing(5),
                container(
                    row![
                        button(text("No").size(16)).on_press(Message::NoOverwrite),
                        button(text("Yes").size(16)).on_press(Message::Overwrite),
                    ]
                    .spacing(20)
                )
                .center_x()
            ]
            .spacing(10)]
            .spacing(20),
        )
        .width(Length::Shrink)
        .padding(15)
        .style(theme::Container::Box)
    }
}

impl Application for Picture {
    type Message = Message;
    type Flags = PictureFlags;
    type Theme = iced::Theme;
    type Executor = executor::Default;

    fn new(flags: PictureFlags) -> (Self, Command<Message>) {
        let mut png_path = PathBuf::from(&flags.file_name);
        png_path.set_extension("png");

        let (svg_tree, pixmap_size) = generate_svg_tree(&flags.svg_content);
        let height = pixmap_size.height();
        let width = pixmap_size.width();
        let ratio = width / height;
        let output_height = pixmap_size.height();
        let output_width = pixmap_size.width();

        let picture = Picture {
            ask_overwrite: false,
            show_modal: false,
            svg_content: flags.svg_content,
            file_name: flags.file_name,
            output_file_name: String::from(png_path.to_str().unwrap()),
            layers: flags.layers.iter().map(|l| (l.to_string(), true)).collect(),
            matrix_transform: (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            svg_tree: Some(svg_tree),
            ratio,
            height,
            width,
            output_height,
            output_width,
            current_scroll: 1.0,
            ..Default::default()
        };
        (
            picture,
            font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(|_| Message::FontLoaded),
        )
    }

    fn title(&self) -> String {
        self.file_name.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, status| {
            if status == iced::event::Status::Captured {
                match event {
                    Mouse(ButtonPressed(Left)) => Some(Message::StartPan),
                    Mouse(ButtonReleased(Left)) => Some(Message::EndPan),
                    Mouse(CursorMoved {
                        position: Point { x, y },
                    }) => Some(Message::CursorMoved(x, y)),
                    Mouse(WheelScrolled {
                        delta: ScrollDelta::Lines { x: _, y },
                    }) => Some(Message::Scroll(y)),
                    _ => None,
                }
            } else {
                match event {
                    Window(_, window::Event::Resized { width, height }) => {
                        Some(Message::Resized(width, height))
                    }
                    _ => None,
                }
            }
        })
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::FontLoaded => Command::none(),
            Message::ToggleLayerVisibility(layer, visible) => {
                self.layers = self
                    .layers
                    .iter()
                    .map(|(l, c)| {
                        let current = l.to_string();
                        if current == layer {
                            (current, visible)
                        } else {
                            (current, *c)
                        }
                    })
                    .collect();
                self.svg_content = set_visible_layers(&self.svg_content, &self.layers);
                Command::none()
            }
            Message::OpenExport => {
                self.show_modal = true;
                Command::none()
            }
            Message::OutFileName(output_file_name) => {
                self.output_file_name = output_file_name;
                Command::none()
            }
            Message::OutHeight(height) => {
                self.output_height = height;
                self.output_width = height * self.ratio;
                Command::none()
            }
            Message::OutWidth(width) => {
                self.output_width = width;
                self.output_height = width / self.ratio;
                Command::none()
            }
            Message::CancelExport => {
                self.show_modal = false;
                self.output_height = self.height;
                self.output_width = self.width;
                Command::none()
            }
            Message::SaveExport => {
                let scale = self.output_width / self.width;
                let png_content = generate_png(
                    self.svg_tree.as_ref().unwrap(),
                    &Size::from_wh(self.output_width, self.output_height).unwrap(),
                    scale,
                );
                self.png_content = png_content;
                let exported = export_png(&self.png_content, &self.output_file_name, false);
                if exported.is_none() {
                    self.ask_overwrite = true;
                } else {
                    self.show_modal = false;
                    self.output_height = self.height;
                    self.output_width = self.width;
                }
                Command::none()
            }
            Message::Overwrite => {
                self.show_modal = false;
                self.ask_overwrite = false;
                self.output_height = self.height;
                self.output_width = self.width;
                export_png(&self.png_content, &self.output_file_name, true);
                Command::none()
            }
            Message::NoOverwrite => {
                self.ask_overwrite = false;
                Command::none()
            }
            Message::Scroll(scroll) => {
                self.current_scroll += 0.05 * scroll;
                if self.current_scroll < 1.0 {
                    self.current_scroll = 1.0;
                    self.matrix_transform = (1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
                } else {
                    let scale = 1.0 + 0.05 * scroll;

                    let (a, b, c, d, e, f) = self.matrix_transform;
                    self.matrix_transform = (
                        a * scale,
                        b * scale,
                        c * scale,
                        d * scale,
                        e + (1.0 - scale) * self.width / 2.0,
                        f + (1.0 - scale) * self.height / 2.0,
                    );
                }
                self.svg_content = transform_svg(&self.svg_content, self.matrix_transform);
                Command::none()
            }
            Message::StartPan => {
                self.panning = true;
                Command::none()
            }
            Message::CursorMoved(x, y) => {
                let old_x = self.current_x;
                let old_y = self.current_y;
                self.current_x = (x / self.current_width as f32) * self.width;
                self.current_y = (y / self.current_height as f32) * self.height;
                if self.panning && self.current_scroll > 1.0 {
                    let (a, b, c, d, e, f) = self.matrix_transform;
                    self.matrix_transform = (
                        a,
                        b,
                        c,
                        d,
                        e + self.current_x - old_x,
                        f + self.current_y - old_y,
                    );
                    self.svg_content = transform_svg(&self.svg_content, self.matrix_transform);
                }
                Command::none()
            }
            Message::EndPan => {
                self.panning = false;
                Command::none()
            }
            Message::Reset => {
                self.matrix_transform = (1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
                self.current_scroll = 1.0;
                self.svg_content = transform_svg(&self.svg_content, self.matrix_transform);
                Command::none()
            }
            Message::Resized(width, height) => {
                self.current_width = width;
                self.current_height = height;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let handle = svg::Handle::from_memory(self.svg_content.clone());

        let inner_svg = svg(handle).width(Length::Fill).height(Length::Fill);
        let svg = ExtendedSvg { inner: inner_svg };

        let checkboxes: Vec<Checkbox<Self::Message>> = self
            .layers
            .iter()
            .map(|(l, c)| {
                checkbox(l.as_str(), *c)
                    .on_toggle(|v| Message::ToggleLayerVisibility(l.to_string(), v))
            })
            .collect();

        let mut row = Row::new()
            .spacing(20)
            .align_items(iced::Alignment::Center)
            .width(Length::Fill);

        for checkbox in checkboxes {
            row = row.push(checkbox)
        }

        let export_button = button(text("Export to PNG").size(16)).on_press(Message::OpenExport);
        let reset_button = button(text("Reset zoom/pan").size(16)).on_press(Message::Reset);
        let content = container(
            column![
                svg,
                container(row).width(Length::Fill).center_x(),
                container(row![export_button, reset_button].spacing(50))
                    .width(Length::Fill)
                    .center_x()
            ]
            .spacing(20)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y();

        if self.show_modal {
            if self.ask_overwrite {
                let modal = self.overwrite_modal();
                Modal::new(content, Some(modal)).into()
            } else {
                let modal = self.save_modal();
                Modal::new(content, Some(modal)).into()
            }
        } else {
            content.into()
        }
    }
}
