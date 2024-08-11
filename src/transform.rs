use std::borrow::Cow;
use std::io::Cursor;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

pub(crate) fn transform_svg(
    svg_content: &Vec<u8>,
    matrix_transform: (f32, f32, f32, f32, f32, f32),
) -> Vec<u8> {
    let mut reader = Reader::from_reader(svg_content.as_slice());
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) if element.name().as_ref() == b"svg" => {
                let mut new_svg = BytesStart::new("svg");

                let has_transform = element.attributes().any(|a| {
                    let attr = a.as_ref().unwrap();
                    attr.key.as_ref() == b"transform"
                });

                let (a, b, c, d, e, f) = matrix_transform;
                let attr_transform = format!("matrix({a} {b} {c} {d} {e} {f})");

                let current_attributes = element.attributes();
                if !has_transform {
                    new_svg.extend_attributes(current_attributes.map(|a| a.unwrap()));
                    new_svg.push_attribute(("transform", attr_transform.as_ref()));
                } else {
                    new_svg.extend_attributes(current_attributes.map(|a| {
                        let attr = a.unwrap();
                        if attr.key.as_ref() == b"transform" {
                            Attribute {
                                key: attr.key,
                                value: Cow::from(attr_transform.as_bytes()),
                            }
                        } else {
                            attr
                        }
                    }));
                }
                writer.write_event(Event::Start(new_svg)).unwrap()
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
