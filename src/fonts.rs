use std::borrow::Cow;
use std::io::Cursor;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

pub(crate) fn replace_fonts(svg_content: Vec<u8>) -> Vec<u8> {
    let mut reader = Reader::from_reader(svg_content.as_slice());
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"text" => {
                let mut current_text = BytesStart::new("text");
                let current_attributes = e.attributes();
                current_text.extend_attributes(current_attributes.map(|a| {
                    let attr = a.unwrap();
                    if attr.key.as_ref() == b"font-family" {
                        convert_font(attr)
                    } else {
                        attr
                    }
                }));
                writer.write_event(Event::Start(current_text)).unwrap()
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e).unwrap();
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
        }
    }

    writer.into_inner().into_inner()
}

fn convert_font(attr: Attribute) -> Attribute {
    match attr.value.as_ref() {
        b"Helvetica" => Attribute {
            key: attr.key,
            value: Cow::from(b"sans-serif"),
        },
        _ => attr,
    }
}
