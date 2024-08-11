use core::f32;
use std::path::PathBuf;

use crate::export::{export_png, generate_png, generate_svg_tree};
use crate::layers::set_visible_layers;
use iced::widget::{
    button, checkbox, column, container, row, svg, text, text_input, Checkbox, Container, Row,
};
use iced::{executor, font, theme, Application, Command};
use iced::{Element, Length};
use iced_aw::number_input;
use iced_aw::widgets::Modal;
use resvg::usvg::{Size, Tree};

#[derive(Debug, Default)]
pub(crate) struct Picture {
    // graphical properties
    ask_overwrite: bool,
    show_modal: bool,

    // content + layers
    file_name: String,
    svg_content: Vec<u8>,
    layers: Vec<(String, bool)>,

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
    ToggleLayerVisibility(String, bool),
    CancelExport,
    OutFileName(String),
    OutWidth(f32),
    OutHeight(f32),
    OpenExport,
    SaveExport,
    Overwrite,
    NoOverwrite,
    FontLoaded,
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

        let picture = Picture {
            ask_overwrite: false,
            show_modal: false,
            svg_content: flags.svg_content,
            file_name: flags.file_name,
            output_file_name: String::from(png_path.to_str().unwrap()),
            layers: flags.layers.iter().map(|l| (l.to_string(), true)).collect(),
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
                // generate tree first time export dialog is opened
                if self.svg_tree.is_none() {
                    let (svg_tree, pixmap_size) = generate_svg_tree(&self.svg_content);
                    self.svg_tree = Some(svg_tree);
                    self.height = pixmap_size.height();
                    self.width = pixmap_size.width();
                    self.ratio = self.width / self.height;
                    self.output_height = pixmap_size.height();
                    self.output_width = pixmap_size.width();
                }
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
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let handle = svg::Handle::from_memory(self.svg_content.clone());

        let svg = svg(handle).width(Length::Fill).height(Length::Fill);

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

        let content = container(
            column![
                svg,
                container(row).width(Length::Fill).center_x(),
                container(export_button).width(Length::Fill).center_x()
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
