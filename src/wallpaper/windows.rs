use crate::error::LRatio;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

const SPI_SETDESKWALLPAPER: u32 = 0x0014;
const SPIF_UPDATEINIFILE: u32 = 0x01;
const SPIF_SENDCHANGE: u32 = 0x02;

#[allow(non_snake_case)]
extern "system" {
    fn SystemParametersInfoW(uiAction: u32, uiParam: u32, pvParam: *const u16, fWinIni: u32)
    -> i32;
}

pub fn set(path: &Path) -> Result<(), LRatio> {
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide.as_ptr(),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };

    if result != 0 {
        Ok(())
    } else {
        Err(LRatio::WallpaperSet("SystemParametersInfoW failed".into()))
    }
}
