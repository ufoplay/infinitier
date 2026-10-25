use slint::ComponentHandle;

use crate::view::rows_model;
use crate::viewers::fnt::{Field, FntViewer};
use crate::{AppWindow, FntAdapter, FntField};

pub fn refresh(ui: &AppWindow, viewer: &FntViewer) {
    let adapter = ui.global::<FntAdapter>();
    let fields: Vec<FntField> = viewer
        .fields()
        .into_iter()
        .map(
            |Field {
                 attribute,
                 value,
                 offset,
                 size,
             }| FntField {
                attribute: attribute.into(),
                value: value.into(),
                offset: offset.into(),
                size: size.into(),
            },
        )
        .collect();
    adapter.set_fields(rows_model(fields));
    adapter.set_note(viewer.note().unwrap_or_default().into());
    adapter.set_dump_summary(viewer.dump_summary().into());
    adapter.set_dump(viewer.dump().into());
}
