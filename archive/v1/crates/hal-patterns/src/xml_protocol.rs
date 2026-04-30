pub struct XmlPatch {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl XmlPatch {
    pub fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    pub fn to_xml(&self) -> String {
        format!(
            "<patch><r>{}</r><g>{}</g><b>{}</b></patch>",
            self.r, self.g, self.b
        )
    }
}

pub struct XmlPattern {
    pub name: String,
    pub chapter: u32,
}

impl XmlPattern {
    pub fn to_xml(&self) -> String {
        format!(
            "<pattern><name>{}</name><chapter>{}</chapter></pattern>",
            self.name, self.chapter
        )
    }
}
