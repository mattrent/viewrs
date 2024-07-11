use std::borrow::Cow;
use std::io::Cursor;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

pub(crate) fn get_layers(svg_content: &Vec<u8>) -> Vec<String> {
    let mut reader = Reader::from_reader(svg_content.as_slice());
    reader.config_mut().trim_text(true);

    let mut groups: Vec<BytesStart> = Vec::new();

    loop {
        match reader.read_event() {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"g" {
                    groups.push(e)
                }
            }

            _ => (),
        }
    }

    let layers: Vec<String> = get_layers_from_groups(groups)
        .iter()
        .map(|l| extract_layer_name(l))
        .flatten()
        .collect();

    layers
}

pub(crate) fn set_visible_layers(svg_content: &Vec<u8>, layers: &Vec<(String, bool)>) -> Vec<u8> {
    let mut reader = Reader::from_reader(svg_content.as_slice());
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"g" && is_layer(&e) => {
                let mut current_layer = BytesStart::new("g");

                let (_, visibility) = layers
                    .iter()
                    .find(|(l, _)| *l == extract_layer_name(&e).expect("layer with no name"))
                    .expect("mismatched window-svg layers");
                let attr_visibility = if *visibility { "visible" } else { "hidden" };

                let has_visibility = e.attributes().any(|a| {
                    let attr = a.as_ref().unwrap();
                    attr.key.as_ref() == b"visibility"
                });

                let current_attributes = e.attributes();
                if !has_visibility {
                    current_layer.extend_attributes(current_attributes.map(|a| a.unwrap()));
                    current_layer.push_attribute(("visibility", attr_visibility));
                } else {
                    current_layer.extend_attributes(current_attributes.map(|a| {
                        let attr = a.unwrap();
                        if attr.key.as_ref() == b"visibility" {
                            Attribute {
                                key: attr.key,
                                value: Cow::from(attr_visibility.as_bytes()),
                            }
                        } else {
                            attr
                        }
                    }));
                }
                writer.write_event(Event::Start(current_layer)).unwrap()
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

fn is_layer(group: &BytesStart) -> bool {
    group.attributes().any(|a| match a {
        Ok(attr) => attr.key.as_ref() == b"inkscape:groupmode" && attr.value.as_ref() == b"layer",
        Err(_) => false,
    })
}

fn get_layers_from_groups(groups: Vec<BytesStart>) -> Vec<BytesStart> {
    groups.into_iter().filter(|g| is_layer(g)).collect()
}

fn extract_layer_name(event: &BytesStart) -> Option<String> {
    let name = event
        .attributes()
        .find(|a| match a {
            Err(_) => false,
            Ok(attr) => attr.key.as_ref() == b"inkscape:label",
        })?
        .ok()?;
    String::from_utf8(name.value.as_ref().to_owned())
        .map(|s| s.replace("&quot;", "\""))
        .ok()
}
