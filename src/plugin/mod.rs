use std::fs;
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
}
