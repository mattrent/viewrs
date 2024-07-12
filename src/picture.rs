use crate::layers::set_visible_layers;
use iced::widget::{checkbox, column, container, svg, Checkbox, Row};
use iced::{executor, Application, Command};
use iced::{Element, Length};

#[derive(Debug, Default)]
pub(crate) struct Picture {
    file_name: String,
    svg_content: Vec<u8>,
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
}

impl Application for Picture {
    type Message = Message;
    type Flags = PictureFlags;
    type Theme = iced::Theme;
    type Executor = executor::Default;

    fn new(flags: PictureFlags) -> (Self, Command<Message>) {
        let picture = Picture {
            svg_content: flags.svg_content,
            file_name: flags.file_name,
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
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let handle = svg::Handle::from_memory(self.svg_content.clone());

        let svg = svg(handle).width(Length::Fill).height(Length::Fill);

        let buttons: Vec<Checkbox<Self::Message>> = self
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

        for checkbox in buttons {
            row = row.push(checkbox)
        }

        container(
            column![svg, container(row).width(Length::Fill).center_x()]
                .spacing(20)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
    }
}
