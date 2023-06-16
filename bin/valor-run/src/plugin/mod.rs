use std::{collections::HashMap, fs};

#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    content: Vec<u8>,
    pub active: bool,
}

impl Plugin {
    pub fn new(name: String, active: bool) -> Self {
        let content = fs::read(&name).expect("Epic Fail!, The file doesn't exist!. :(");
        Plugin {
            name,
            content,
            active,
        }
    }

    pub fn get_plugin(&self) -> &[u8] {
        &self.content
    }

    // ToDo: Try this in an example or something similar
    #[allow(dead_code)]
    pub fn new_map(paths: Vec<String>, all_active: bool) -> HashMap<String, Self> {
        let mut map_plugins = HashMap::<String, Self>::new();
        for path in paths {
            let plugin = self::Plugin::new(path.clone(), all_active);
            map_plugins.insert(path, plugin);
        }
        map_plugins
    }

    pub fn new_vec(paths: Vec<String>, all_active: bool) -> Vec<Self> {
        let mut vec_plugins: Vec<Self> = Vec::new();
        for path in paths {
            let plugin = self::Plugin::new(path.clone(), all_active);
            vec_plugins.push(plugin);
        }
        vec_plugins
    }
}
