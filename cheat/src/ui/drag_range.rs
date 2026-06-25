use egui::{DragValue, Response, Ui, Widget, emath::Numeric};
use std::ops::RangeInclusive;

pub struct DragRange<'a, T: Numeric> {
    label: &'a str,
    range: &'a mut RangeInclusive<T>,
    clamp: RangeInclusive<T>,
}

impl<'a, T: Numeric> DragRange<'a, T> {
    pub fn new(label: &'a str, range: &'a mut RangeInclusive<T>, clamp: RangeInclusive<T>) -> Self {
        Self {
            label,
            range,
            clamp,
        }
    }
}

impl<'a, T: Numeric> Widget for DragRange<'a, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut start = *self.range.start();
        let mut end = *self.range.end();

        let response = ui
            .horizontal(|ui| {
                let res = ui.add(
                    DragValue::new(&mut start)
                        .range(self.clamp.clone())
                        .suffix("  "),
                );
                let res = ui
                    .add(DragValue::new(&mut end).range(self.clamp.clone()))
                    .union(res);
                ui.label(self.label);
                res
            })
            .inner;

        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        *self.range = start..=end;

        response
    }
}
