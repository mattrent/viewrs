use std::path::PathBuf;

use crate::export::{export_png, generate_png};
use crate::layers::set_visible_layers;
use iced::widget::{
    button, checkbox, column, container, row, svg, text, text_input, Checkbox, Container, Row,
};
use iced::{executor, theme, Application, Command};
use iced::{Element, Length};
use iced_aw::widgets::Modal;

#[derive(Debug, Default)]
pub(crate) struct Picture {
    ask_overwrite: bool,
    show_modal: bool,
    file_name: String,
    output_file_name: String,
    svg_content: Vec<u8>,
    png_content: Vec<u8>,
    layers: Vec<(String, bool)>,
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
    OpenExport,
    SaveExport,
    Overwrite,
    NoOverwrite,
}

impl Picture {
    fn main_modal(&self) -> Container<Message> {
        container(
            column![column![
                column![text("Path").size(12),].spacing(5),
                text_input("", &self.output_file_name).on_input(Message::OutFileName),
                container(
                    row![
                        button(text("Cancel")).on_press(Message::CancelExport),
                        button(text("Save")).on_press(Message::SaveExport),
                    ]
                    .spacing(20)
                )
                .width(Length::Fill)
                .center_x()
            ]
            .spacing(10)]
            .spacing(20),
        )
        .width(Length::Shrink)
        .padding(10)
        .style(theme::Container::Box)
    }

    fn overwrite_modal(&self) -> Container<Message> {
        container(
            column![column![
                column![text("File exists. Overwrite?").size(12),].spacing(5),
                container(
                    row![
                        button(text("No")).on_press(Message::NoOverwrite),
                        button(text("Yes")).on_press(Message::Overwrite),
                    ]
                    .spacing(20)
                )
                .width(Length::Fill)
                .center_x()
            ]
            .spacing(10)]
            .spacing(20),
        )
        .width(Length::Shrink)
        .padding(10)
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
        (picture, Command::none())
    }

    fn title(&self) -> String {
        self.file_name.clone()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
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
            Message::CancelExport => {
                self.show_modal = false;
                Command::none()
            }
            Message::SaveExport => {
                let png_content = generate_png(&self.svg_content);
                self.png_content = png_content;
                let exported = export_png(&self.png_content, &self.output_file_name, false);
                if exported == None {
                    self.ask_overwrite = true;
                } else {
                    self.show_modal = false;
                }
                Command::none()
            }
            Message::Overwrite => {
                self.show_modal = false;
                self.ask_overwrite = false;
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
                let modal = self.main_modal();
                Modal::new(content, Some(modal)).into()
            }
        } else {
            content.into()
        }
    }
}
