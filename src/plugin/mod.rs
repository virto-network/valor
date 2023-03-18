#[derive(Debug)]
pub struct Plugin<'a> {
    name: &'a str,
    content: Vec<u8>,
}

impl<'a> Plugin<'a> {
    pub fn new(name: &'a str, content: Vec<u8>) -> Self {
        Plugin { name, content }
    }
    pub fn get_plugin(&self) -> &[u8] {
        &self.content
    }
}
