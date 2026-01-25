use gpui::*;
use super::data::SignalSeries;

pub struct ChartRenderer {
    series: Vec<SignalSeries>,
}

impl ChartRenderer {
    pub fn new(series: Vec<SignalSeries>) -> Self {
        Self { series }
    }

    pub fn render(self) -> impl IntoElement {
        let count = self.series.len();
        let points = self.series.iter().map(|s| s.points.len()).sum::<usize>();
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .bg(rgb(0x101010))
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(0xffffff))
                    .child("Signal Chart")
            )
            .child(
                div()
                    .text_color(rgb(0x9ca3af))
                    .child(format!("Series: {}, Total Points: {}", count, points))
            )
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .text_color(rgb(0x606060))
                    .child("(Canvas rendering disabled temporarily for compilation fix)")
            )
    }
}
