use std::collections::HashMap;

use crate::os::process::Process;

pub struct Schema {
    scopes: HashMap<String, ModuleScope>,
}

impl Schema {
    pub fn new(process: &Process, schema_module: usize) -> Option<Self> {
        let schema_system =
            process.scan("48 8b 05 ? ? ? ? 48 8d 15 ? ? ? ? 4c 89 e9", schema_module)?;
        let schema_system = process.get_relative_address(schema_system, 3, 7);
        let schema_system: usize = process.read(schema_system);

        let type_scopes_len: i32 = process.read(schema_system + 0x1F0);
        let type_scopes_vec: usize = process.read(schema_system + 0x1F8);
        let mut scopes = HashMap::new();
        for i in 0..type_scopes_len as usize {
            let type_scope_address = process.read(type_scopes_vec + (i * 8));
            let type_scope = ModuleScope::new(process, type_scope_address);
            scopes.insert(type_scope.name.clone(), type_scope);
        }

        Some(Self { scopes })
    }

    #[allow(unused)]
    pub fn get(&self, library: &str, class: &str, field: &str) -> Option<usize> {
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
    fn new(process: &Process, address: usize) -> Self {
        let name = process.read_string_uncached(address + 0x08);

        let mut classes = HashMap::new();
        // has 1024 buckets
        let hash_vector = address + 0x560 + 0x90;
        for i in 0..1024 {
            // first_uncomitted
            let mut current_element: usize = process.read(hash_vector + (i * 24) + 0x28);

            while current_element != 0 {
                let data: usize = process.read(current_element + 0x10);
                if data != 0 {
                    let class = Class::new(process, data);
                    classes.insert(class.name.clone(), class);
                }
                current_element = process.read(current_element + 0x08);
            }
        }

        // free_list (HashAllocatedBlob)
        let mut current_blob: usize = process.read(address + 0x560 + 0x20);
        while current_blob != 0 {
            let data: usize = process.read(current_blob + 0x10);
            if data > process.min && data < process.max {
                let class = Class::new(process, data);
                classes.insert(class.name.clone(), class);
            }
            current_blob = process.read(current_blob);
        }

        Self { name, classes }
    }

    pub fn get(&self, class: &str, field: &str) -> Option<usize> {
        let Some(c) = self.classes.get(class) else {
            utils::warn!("could not find class {class}");
            return None;
        };
        let f = c.get(field);
        if f.is_none() {
            utils::warn!("could not find field {field} in class {class}");
        }
        f
    }

    pub fn get_class(&self, class: &str) -> Option<&Class> {
        self.classes.get(class)
    }
}

pub struct Class {
    name: String,
    fields: HashMap<String, usize>,
    size: i32,
}

impl Class {
    fn new(process: &Process, address: usize) -> Self {
        let name = process.read_string_uncached(process.read(address + 0x08));

        let mut fields = HashMap::new();
        let field_count: i16 = process.read(address + 0x24);
        let size = process.read(address + 0x20);
        if !(0..=20000).contains(&field_count) {
            return Self { name, fields, size };
        }
        let fields_vec: usize = process.read(address + 0x30);
        for i in 0..field_count as usize {
            let field = Field::new(process, fields_vec + (0x20 * i));
            fields.insert(field.name, field.offset);
        }
        Self { name, fields, size }
    }

    fn get(&self, field: &str) -> Option<usize> {
        self.fields.get(field).copied()
    }

    pub fn size(&self) -> i32 {
        self.size
    }
}

struct Field {
    name: String,
    offset: usize,
}

impl Field {
    fn new(process: &Process, address: usize) -> Self {
        let name = process.read_string_uncached(process.read(address));
        let offset = process.read::<i32>(address + 0x10) as usize;

        Self { name, offset }
    }
}
