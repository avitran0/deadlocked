use std::collections::HashMap;

use utils::log;

use crate::os::process::Process;

pub struct Schema {
    scopes: HashMap<String, ModuleScope>,
}

impl Schema {
    pub fn new(process: &Process, schema_module: u64) -> Option<Self> {
        let schema_system = process.scan(
            "48 8D 3D ? ? ? ? E8 ? ? ? ? 48 8B BD ? ? ? ? 31 F6 E8 ? ? ? ? E9",
            schema_module,
        )?;
        let schema_system = process.get_relative_address(schema_system, 3, 7);

        let type_scopes_len: i32 = process.read(schema_system + 0x1F0);
        let type_scopes_vec: u64 = process.read(schema_system + 0x1F8);
        let mut scopes = HashMap::new();
        for i in 0..type_scopes_len as u64 {
            let type_scope_address = process.read(type_scopes_vec + (i * 8));
            let type_scope = ModuleScope::new(process, type_scope_address);
            scopes.insert(type_scope.name.clone(), type_scope);
        }

        Some(Self { scopes })
    }

    #[allow(unused)]
    pub fn get(&self, library: &str, class: &str, field: &str) -> Option<u64> {
        let scope = self.scopes.get(library)?;
        let class = scope.get_class(class)?;
        class.get(field)
    }

    pub fn get_library(&self, library: &str) -> Option<&ModuleScope> {
        self.scopes.get(library)
    }
}

pub struct ModuleScope {
    name: String,
    classes: HashMap<String, Class>,
}

impl ModuleScope {
    fn new(process: &Process, address: u64) -> Self {
        let name = process.read_string_uncached(address + 0x08);

        let mut classes = HashMap::new();
        // has 1024 buckets
        let hash_vector = address + 0x560 + 0x90;
        for i in 0..1024 {
            // first_uncomitted
            let mut current_element: u64 = process.read(hash_vector + (i * 24) + 0x28);

            while current_element != 0 {
                let data: u64 = process.read(current_element + 0x10);
                if data != 0 {
                    let class = Class::new(process, data);
                    classes.insert(class.name.clone(), class);
                }
                current_element = process.read(current_element + 0x08);
            }
        }

        // free_list (HashAllocatedBlob)
        let mut current_blob: u64 = process.read(address + 0x560 + 0x20);
        while current_blob != 0 {
            let data: u64 = process.read(current_blob + 0x10);
            if data > process.min && data < process.max {
                let class = Class::new(process, data);
                classes.insert(class.name.clone(), class);
            }
            current_blob = process.read(current_blob);
        }

        Self { name, classes }
    }

    pub fn get(&self, class: &str, field: &str) -> Option<u64> {
        let Some(c) = self.classes.get(class) else {
            log::warn!("could not find class {class}");
            return None;
        };
        let f = c.get(field);
        if f.is_none() {
            log::warn!("could not find field {field} in class {class}");
        }
        f
    }

    pub fn try_get(&self, class: &str, field: &str) -> Option<u64> {
        self.classes.get(class)?.get(field)
    }

    pub fn find_in_classes_containing(
        &self,
        class_fragments: &[&str],
        fields: &[&str],
    ) -> Option<u64> {
        for (class_name, class) in &self.classes {
            if !class_fragments
                .iter()
                .any(|fragment| class_name.contains(fragment))
            {
                continue;
            }
            for field in fields {
                if let Some(offset) = class.get(field) {
                    return Some(offset);
                }
            }
        }
        None
    }

    pub fn get_class(&self, class: &str) -> Option<&Class> {
        self.classes.get(class)
    }
}

pub struct Class {
    name: String,
    fields: HashMap<String, u64>,
    size: i32,
}

impl Class {
    fn new(process: &Process, address: u64) -> Self {
        let name = process.read_string_uncached(process.read(address + 0x08));

        let mut fields = HashMap::new();

        let mut selected: Option<(i16, i32, u64, i32)> = None;
        for (field_count_offset, size_offset, fields_vec_offset) in
            [(0x1C, 0x18, 0x28), (0x24, 0x20, 0x30)]
        {
            let field_count: i16 = process.read(address + field_count_offset);
            if !(0..=20000).contains(&field_count) {
                continue;
            }

            let size: i32 = process.read(address + size_offset);
            let fields_vec: u64 = process.read(address + fields_vec_offset);
            if field_count > 0 && !(process.min..=process.max).contains(&fields_vec) {
                continue;
            }

            let sample_count = field_count.min(4) as u64;
            let mut score = if field_count == 0 { -1 } else { 0 };
            for i in 0..sample_count {
                let field_address = fields_vec + (0x20 * i);
                let field_name_address: u64 = process.read(field_address);
                if !(process.min..=process.max).contains(&field_name_address) {
                    continue;
                }
                let field_name = process.read_string_uncached(field_name_address);
                if !field_name.is_empty() {
                    score += 1;
                }
                if field_name.starts_with("m_") {
                    score += 2;
                }
            }

            match selected {
                None => selected = Some((field_count, size, fields_vec, score)),
                Some((best_count, _, _, best_score))
                    if score > best_score || (score == best_score && field_count > best_count) =>
                {
                    selected = Some((field_count, size, fields_vec, score))
                }
                _ => {}
            }
        }

        let Some((field_count, size, fields_vec, _)) = selected else {
            return Self {
                name,
                fields,
                size: 0,
            };
        };

        for i in 0..field_count as u64 {
            let field = Field::new(process, fields_vec + (0x20 * i));
            fields.insert(field.name, field.offset);
        }
        Self { name, fields, size }
    }

    fn get(&self, field: &str) -> Option<u64> {
        self.fields.get(field).copied()
    }

    pub fn size(&self) -> i32 {
        self.size
    }
}

struct Field {
    name: String,
    offset: u64,
}

impl Field {
    fn new(process: &Process, address: u64) -> Self {
        let name = process.read_string_uncached(process.read(address));
        let offset = process.read::<i32>(address + 0x10) as u64;

        Self { name, offset }
    }
}
