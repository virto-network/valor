use std::{collections::HashMap, fs};

#[derive(Debug)]
pub struct Plugin<'a> {
    pub name: &'a str,
    content: Vec<u8>,
    pub active: bool,
}

impl<'a> Plugin<'a> {
    pub fn new(name: &'a str, active: bool) -> Self {
        let content = fs::read(name).expect("Epic Fail!, The file doesn't exist!. :(");
        Plugin {
            name,
            content,
            active,
        }
    }

    pub fn get_plugin(&self) -> &[u8] {
        &self.content
    }

    pub fn new_map(paths: &'a Vec<String>, all_active: bool) -> HashMap<&str, Self> {
        let mut map_plugins = HashMap::<&str, Self>::new();
        for path in paths.iter() {
            let active = if all_active { true } else { false };
            let plugin = self::Plugin::new(path.as_str(), active);
            map_plugins.insert(path.as_str(), plugin);
        }
        map_plugins
    }

    pub fn new_vec(paths: &'a Vec<String>, all_active: bool) -> Vec<Self> {
        let mut vec_plugins: Vec<Self> = Vec::new();
        for path in paths.iter() {
            let plugin = self::Plugin::new(path.as_str(), all_active);
            vec_plugins.push(plugin);
        }
        vec_plugins
    }
}
