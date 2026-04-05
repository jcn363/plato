#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Serif,
    SansSerif,
    Display,
    Keyboard,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    REGULAR,
    ITALIC,
    BOLD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub family: Family,
    pub variant: Variant,
    pub size: u32,
}
