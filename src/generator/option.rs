pub struct GeneratorOptions {
    pub layer: GenerateLayer,
}

pub enum GenerateLayer {
    Flat,
    Recursive,
}