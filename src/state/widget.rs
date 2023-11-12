use network_tables::Value;

#[derive(Clone, Debug)]
pub struct Widget {
    pub title: String,
    pub value: WidgetKind,
}

#[derive(Clone, Debug)]
pub enum WidgetKind {
    Simple {
        value: Option<Value>,
    },
    Chooser {
        options: Vec<String>,
        default: String,
        active: String,
    },
}
impl WidgetKind {
    pub fn height(&self) -> u16 {
        let raw_height = match self {
            WidgetKind::Simple { .. } => 1,
            WidgetKind::Chooser { options, .. } => options.len() as u16,
        };
        raw_height + 1
    }
}
