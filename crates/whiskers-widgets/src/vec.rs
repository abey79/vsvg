use crate::{Widget, WidgetMapper};
use egui::Ui;

#[derive(Default)]
pub struct VecWidget<T>
where
    T: WidgetMapper<T> + Default,
{
    inner: <T as WidgetMapper<T>>::Type,
}

impl<T> VecWidget<T>
where
    T: WidgetMapper<T> + Default,
{
    pub fn inner(
        mut self,
        update_inner: impl FnOnce(<T as WidgetMapper<T>>::Type) -> <T as WidgetMapper<T>>::Type,
    ) -> Self {
        self.inner = update_inner(self.inner);
        self
    }
}

impl<T> Widget<Vec<T>> for VecWidget<T>
where
    T: WidgetMapper<T> + Default,
{
    fn ui(&self, ui: &mut Ui, label: &str, value: &mut Vec<T>) -> bool {
        let mut changed = false;

        let inner_use_grid = <T as WidgetMapper<T>>::Type::use_grid();

        //TODO: meaningful summary
        crate::collapsing_header(ui, label, "", true, |ui| {
            if inner_use_grid {
                egui::Grid::new(label).num_columns(2).show(ui, |ui| {
                    for (i, item) in value.iter_mut().enumerate() {
                        let item_label = format!("[{i}]:");
                        changed |= self.inner.ui(ui, &item_label, item);
                        ui.end_row();
                    }
                });
            } else {
                for (i, item) in value.iter_mut().enumerate() {
                    let item_label = format!("[{i}]:");
                    changed |= self.inner.ui(ui, &item_label, item);
                }
            }
        });

        changed
    }

    fn use_grid() -> bool {
        false
    }
}

impl<T> WidgetMapper<Vec<T>> for Vec<T>
where
    T: WidgetMapper<T> + Default,
{
    type Type = VecWidget<T>;
}
