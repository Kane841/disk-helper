use crate::error::{err, AppError, ErrorCode};
use crate::models::VolumeInfo;

const C_DRIVE: &str = "C:";

fn round_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn build_volume_info(total_bytes: u64, free_bytes: u64) -> VolumeInfo {
    let used_bytes = total_bytes.saturating_sub(free_bytes);
    let usage_percent = if total_bytes > 0 {
        round_one_decimal((used_bytes as f64 / total_bytes as f64) * 100.0)
    } else {
        0.0
    };

    VolumeInfo {
        drive: C_DRIVE.into(),
        total_bytes,
        used_bytes,
        free_bytes,
        usage_percent,
    }
}

#[cfg(windows)]
pub fn get_c_drive_volume() -> Result<VolumeInfo, AppError> {
    use windows::core::w;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let mut free_available = 0u64;
    let mut total_bytes = 0u64;
    let mut total_free = 0u64;

    unsafe {
        GetDiskFreeSpaceExW(
            w!(r"C:\"),
            Some(&mut free_available),
            Some(&mut total_bytes),
            Some(&mut total_free),
        )
        .map_err(|e| {
            err(
                ErrorCode::InternalError,
                format!("GetDiskFreeSpaceExW failed: {e}"),
            )
            .with_target("volume")
        })?;
    }

    let _ = free_available;
    Ok(build_volume_info(total_bytes, total_free))
}

#[cfg(not(windows))]
pub fn get_c_drive_volume() -> Result<VolumeInfo, AppError> {
    Err(
        err(
            ErrorCode::InternalError,
            "VolumeService is only supported on Windows",
        )
        .with_target("volume"),
    )
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn get_c_drive_returns_valid_volume() {
        let volume = get_c_drive_volume().expect("C: volume");
        assert_eq!(volume.drive, "C:");
        assert!(volume.total_bytes > 0);
        assert!(volume.free_bytes <= volume.total_bytes);
        assert!(volume.used_bytes <= volume.total_bytes);
        assert!((0.0..=100.0).contains(&volume.usage_percent));
    }
}
