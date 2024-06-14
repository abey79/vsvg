use crate::{Widget, WidgetMapper};
use egui::Ui;

#[derive(Default)]
pub struct VecWidget<T>
where
    T: WidgetMapper<T> + Default,
{
    inner: T::Type,
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

        let summary = format!(
            "{} item{}",
            value.len(),
            if value.len() > 1 { "s" } else { "" }
        );
        crate::collapsing_header(ui, label, summary, true, |ui| {
            if !value.is_empty() {
                let mut items_to_delete = Vec::new();
                egui::Grid::new(label)
                    .num_columns(if inner_use_grid { 3 } else { 2 })
                    .show(ui, |ui| {
                        for (i, item) in value.iter_mut().enumerate() {
                            let item_label = format!("[{i}]:");

                            if inner_use_grid {
                                changed |= self.inner.ui(ui, &item_label, item);
                            } else {
                                ui.vertical(|ui| {
                                    changed |= self.inner.ui(ui, &item_label, item);
                                });
                            }

                            ui.horizontal_top(|ui| {
                                if ui.button("–").clicked() {
                                    items_to_delete.push(i);
                                }
                            });
                            ui.end_row();
                        }
                    });

                for item in items_to_delete.iter().rev() {
                    value.remove(*item);
                    changed = true;
                }
            }

            if ui.button("+").clicked() {
                value.push(T::default());
                changed = true;
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
