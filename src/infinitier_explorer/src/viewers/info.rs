//! The facts every viewer's info bar shows.

use std::time::Duration;

use bytesize::ByteSize;
use infinitier_core::game::{DataOrigin, GameResource};

/// The resource's size on disk, `? B` when unknown.
pub fn size_label(resource: &GameResource) -> String {
    match resource.file_size {
        Some(size) => ByteSize(size).to_string(),
        None => "? B".to_owned(),
    }
}

/// Where the resource was read from.
pub fn origin_label(resource: &GameResource) -> String {
    match &resource.data_origin {
        DataOrigin::Bif { name } => format!("BIF: {name}"),
        DataOrigin::Dir { name, path } => format!("{name}: {}", path.path().display()),
        DataOrigin::Unhardcoded { folder } => format!("unhardcoded/{folder}"),
        DataOrigin::Missing => "Missing".to_owned(),
    }
}

/// `m:ss.ss`.
pub fn format_duration(d: Duration) -> String {
    let total = d.as_secs_f64();
    let mins = (total / 60.0).floor() as u64;
    let secs = total - (mins as f64) * 60.0;
    format!("{mins}:{secs:05.2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_show_minutes_and_hundredths() {
        assert_eq!(format_duration(Duration::ZERO), "0:00.00");
        assert_eq!(format_duration(Duration::from_millis(83_250)), "1:23.25");
    }
}
