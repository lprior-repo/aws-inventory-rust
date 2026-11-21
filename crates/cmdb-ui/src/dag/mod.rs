pub mod builder;
pub mod graph;
pub mod layout;
pub mod view;
pub mod widget;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod layout_unit_tests;

pub fn script_mod(vm: &mut makepad_widgets::ScriptVm) {
    widget::script_mod(vm);
}
