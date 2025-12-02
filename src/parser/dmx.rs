use std::{collections::HashMap, io::Read};

use glam::Vec3;

use super::binary::{read, read_bytes, read_string};

#[derive(Debug, Clone)]
pub struct Element {
    pub kind: String,
    pub name: String,
    pub attributes: HashMap<String, Attribute>,
}

impl Element {
    pub fn new(kind: String, name: String) -> Self {
        Self {
            kind,
            name,
            attributes: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, attribute: Attribute) {
        self.attributes.insert(name, attribute);
    }
}

#[derive(Debug, Clone)]
pub enum Attribute {
    Vec3Array(Vec<Vec3>),
    IntegerArray(Vec<i32>),
}

pub fn parse_dmx(reader: &mut impl Read) -> HashMap<String, Element> {
    let _header = read_string(reader);
    let _prefix_elements: i32 = read(reader);
    let string_count: i32 = read(reader);
    let mut strings = Vec::with_capacity(string_count as usize);
    for _ in 0..string_count {
        strings.push(read_string(reader));
    }

    let element_count: i32 = read(reader);
    let mut elements = Vec::with_capacity(element_count as usize);
    for _ in 0..element_count {
        let element = read_element(reader, &strings);
        elements.push(element);
    }

    for element in &mut elements {
        let attribute_count: i32 = read(reader);
        for _ in 0..attribute_count {
            let name = &strings[read::<i32>(reader) as usize];
            let kind: u8 = read(reader);
            use Attribute as AT;
            let value = match kind {
                34 => AT::IntegerArray({
                    let count: i32 = read(reader);
                    (0..count).map(|_| read(reader)).collect()
                }),
                42 => AT::Vec3Array({
                    let count: i32 = read(reader);
                    (0..count).map(|_| read(reader)).collect()
                }),
                _ => panic!("Unsupported attribute type: {}", kind),
            };
            element.add(name.to_string(), value);
        }
    }
    let mut elems = HashMap::new();
    elements.into_iter().for_each(|e| {
        let name = format!("{}_{}", e.kind, e.name);
        elems.insert(name, e);
    });

    elems
}

fn read_element(reader: &mut impl Read, strings: &[String]) -> Element {
    let kind = &strings[read::<i32>(reader) as usize];
    let name = &strings[read::<i32>(reader) as usize];
    let _uuid = read_bytes(reader, 16);
    Element::new(kind.to_string(), name.to_string())
}
