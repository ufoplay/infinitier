use slint::ComponentHandle;

use crate::view::rows_model;
use crate::viewers::ttf::{SAMPLE_SIZES, TtfViewer, sample_label};
use crate::{AppWindow, TtfAdapter, TtfDetail, TtfSample};

pub fn refresh(ui: &AppWindow, viewer: &TtfViewer, sample: &str) {
    let adapter = ui.global::<TtfAdapter>();
    adapter.set_family(viewer.family().into());
    adapter.set_full_name(viewer.full_name().into());
    let details: Vec<TtfDetail> = viewer
        .details()
        .into_iter()
        .map(|(label, value)| TtfDetail {
            label: label.into(),
            value: value.into(),
        })
        .collect();
    adapter.set_details(rows_model(details));
    adapter.set_sample(sample.into());
    let samples: Vec<TtfSample> = SAMPLE_SIZES
        .into_iter()
        .map(|size| TtfSample {
            label: sample_label(size).into(),
            size,
        })
        .collect();
    adapter.set_samples(rows_model(samples));
}
