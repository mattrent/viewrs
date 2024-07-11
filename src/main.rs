mod layers;

use std::io::Read;
use std::{env, fs, vec};

use iced::widget::{checkbox, column, container, svg, Checkbox, Row};
use iced::{executor, Application, Command};
use iced::{Element, Length, Settings};
use layers::{get_layers, set_visible_layers};

pub fn main() -> iced::Result {
    let args: Vec<_> = env::args().collect();
    let (svg_content, file_name): (Vec<u8>, String) = if args.len() > 1 {
        let file_name = &args[1];
        let mut file = fs::File::open(file_name).expect("unable to open file");
        let metadata = fs::metadata(&file_name).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        file.read(&mut buffer).expect("buffer overflow");
        (buffer, file_name.to_string())
    } else {
        (Vec::new(), String::new())
    };

    let layers = get_layers(&svg_content);

    Picture::run(Settings::with_flags(PictureFlags {
        svg_content,
        file_name,
        layers,
    }))
}

#[derive(Debug, Default)]
struct Picture {
    file_name: String,
    svg_content: Vec<u8>,
    layers: Vec<(String, bool)>,
}

#[derive(Debug, Default)]
struct PictureFlags {
    file_name: String,
    svg_content: Vec<u8>,
    layers: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetSvgContent(Vec<u8>),
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
            Message::SetSvgContent(svg_content) => {
                self.svg_content = svg_content;
                Command::none()
            }
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
                // println!("{}", String::from_utf8(self.svg_content.clone()).unwrap());
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
