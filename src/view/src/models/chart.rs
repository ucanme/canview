use gpui::Hsla;

#[derive(Clone, Copy, Debug)]
pub struct DataPoint {
    pub time: f64,
    pub value: f64,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct Series {
    pub name: String,
    pub unit: Option<String>,
    pub points: Vec<DataPoint>,
    pub color: Hsla,
}
