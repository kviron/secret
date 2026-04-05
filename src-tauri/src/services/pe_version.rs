//! Версия продукта из ресурса версии PE (Windows) — как в свойствах файла / Vortex.

use std::path::Path;

/// `FileVersion`, иначе `ProductVersion` из первого подходящего `required_files` (`.exe`).
#[cfg(windows)]
pub fn pe_file_version_label(install_path: &Path, required_files: &[String]) -> Option<String> {
    imp::pe_file_version_label(install_path, required_files)
}

#[cfg(not(windows))]
pub fn pe_file_version_label(_install_path: &Path, _required_files: &[String]) -> Option<String> {
    None
}

#[cfg(windows)]
mod imp {
    use super::Path;
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };

    fn to_wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn pe_file_version_label(install_path: &Path, required_files: &[String]) -> Option<String> {
        for rel in required_files {
            let p = install_path.join(rel);
            if !p.is_file() {
                continue;
            }
            let is_exe = p
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("exe"));
            if !is_exe {
                continue;
            }
            if let Some(v) = file_version_or_product(&p) {
                let t = v.trim();
                if !t.is_empty() {
                    return Some(t.to_string());
                }
            }
        }
        None
    }

    fn file_version_or_product(path: &Path) -> Option<String> {
        unsafe {
            let wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let size = GetFileVersionInfoSizeW(PCWSTR(wide.as_ptr()), None);
            if size == 0 {
                return None;
            }
            let mut data = vec![0u8; size as usize];
            GetFileVersionInfoW(PCWSTR(wide.as_ptr()), None, size, data.as_mut_ptr().cast())
                .ok()?;

            let trans_key = to_wide_null("\\VarFileInfo\\Translation");
            let mut trans_ptr: *mut c_void = ptr::null_mut();
            let mut trans_len: u32 = 0;
            if !VerQueryValueW(
                data.as_ptr().cast(),
                PCWSTR(trans_key.as_ptr()),
                &mut trans_ptr,
                &mut trans_len,
            )
            .as_bool()
                || trans_ptr.is_null()
                || trans_len < 4
            {
                return None;
            }

            let pair = *(trans_ptr as *const u32);
            let lang = (pair & 0xFFFF) as u16;
            let cp = (pair >> 16) as u16;

            let try_key = |suffix: &str| -> Option<String> {
                let sub = format!("\\StringFileInfo\\{:04x}{:04x}\\{}", lang, cp, suffix);
                let sub_w = to_wide_null(&sub);
                let mut buf: *mut c_void = ptr::null_mut();
                let mut len: u32 = 0;
                if !VerQueryValueW(
                    data.as_ptr().cast(),
                    PCWSTR(sub_w.as_ptr()),
                    &mut buf,
                    &mut len,
                )
                .as_bool()
                    || buf.is_null()
                    || len < 2
                {
                    return None;
                }
                let n_chars = (len as usize) / 2;
                let w = std::slice::from_raw_parts(buf.cast::<u16>(), n_chars);
                let s = String::from_utf16_lossy(w);
                let s = s.trim_end_matches('\0').trim();
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            };

            try_key("FileVersion").or_else(|| try_key("ProductVersion"))
        }
    }
}
