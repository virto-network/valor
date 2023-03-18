use std::{collections::HashMap, fs};
#[derive(Debug)]
pub struct Plugin<'a> {
    name: &'a str,
    content: Vec<u8>,
}

impl<'a> Plugin<'a> {
    pub fn new(name: &'a str) -> Self {
        let content = fs::read(name).expect("Epic Fail!, The file doesn't exist!. :(");
        Plugin { name, content }
    }

    pub fn get_plugin(&self) -> &[u8] {
        &self.content
    }

    pub fn new_map(paths: &'a Vec<String>) -> HashMap<&str, Self> {
        let mut map_plugins = HashMap::<&str, Self>::new();
        for path in paths.iter() {
            let plugin = self::Plugin::new(path.as_str());
            map_plugins.insert(path.as_str(), plugin);
        }
        map_plugins
    }
}
