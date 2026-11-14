//! Small dynamically loaded libmpv client-API adapter. The handle never leaves its worker.
use std::{
    ffi::{CStr, CString, c_char, c_void},
    path::Path,
};
type Handle = *mut c_void;
#[repr(C)]
struct RawEvent {
    id: i32,
    error: i32,
    userdata: u64,
    data: *mut c_void,
}
#[repr(C)]
struct EndFile {
    reason: i32,
    error: i32,
    entry_id: i64,
}
#[repr(C)]
struct StartFile {
    entry_id: i64,
}
pub enum Event {
    Loaded,
    Ended,
    Failed(String),
}
pub struct Mpv {
    handle: Handle,
    command: unsafe extern "C" fn(Handle, *const *const c_char) -> i32,
    set: unsafe extern "C" fn(Handle, *const c_char, i32, *mut c_void) -> i32,
    get: unsafe extern "C" fn(Handle, *const c_char, i32, *mut c_void) -> i32,
    get_string: unsafe extern "C" fn(Handle, *const c_char) -> *mut c_char,
    free: unsafe extern "C" fn(*mut c_void),
    wait: unsafe extern "C" fn(Handle, f64) -> *const RawEvent,
    destroy: unsafe extern "C" fn(Handle),
    error_string: unsafe extern "C" fn(i32) -> *const c_char,
    entry_id: Option<i64>,
    _library: libloading::Library,
}
impl Mpv {
    pub fn open(path: &Path, null_audio: bool) -> Result<Self, String> {
        // Restrict Windows dependency search to the selected runtime directory and trusted defaults.
        #[cfg(windows)]
        let library: libloading::Library =
            unsafe { libloading::os::windows::Library::load_with_flags(path, 0x100 | 0x1000) }
                .map_err(|e| format!("Audio engine unavailable: {e}"))?
                .into();
        #[cfg(not(windows))]
        let library = unsafe { libloading::Library::new(path) }.map_err(|e| e.to_string())?;
        // SAFETY: ABI signatures and layouts match the pinned runtime's include/mpv/client.h.
        // Symbols remain valid because _library outlives handle teardown in Drop.
        unsafe {
            let version = *library
                .get::<unsafe extern "C" fn() -> std::ffi::c_ulong>(b"mpv_client_api_version\0")
                .map_err(|e| e.to_string())?;
            if version() >> 16 != 2 {
                return Err("Unsupported libmpv client API; expected major 2".into());
            }
            let create = *library
                .get::<unsafe extern "C" fn() -> Handle>(b"mpv_create\0")
                .map_err(|e| e.to_string())?;
            let initialize = *library
                .get::<unsafe extern "C" fn(Handle) -> i32>(b"mpv_initialize\0")
                .map_err(|e| e.to_string())?;
            let option = *library
                .get::<unsafe extern "C" fn(Handle, *const c_char, *const c_char) -> i32>(
                    b"mpv_set_option_string\0",
                )
                .map_err(|e| e.to_string())?;
            let mut mpv = Self {
                handle: std::ptr::null_mut(),
                command: *library.get(b"mpv_command\0").map_err(|e| e.to_string())?,
                set: *library
                    .get(b"mpv_set_property\0")
                    .map_err(|e| e.to_string())?,
                get: *library
                    .get(b"mpv_get_property\0")
                    .map_err(|e| e.to_string())?,
                get_string: *library
                    .get(b"mpv_get_property_string\0")
                    .map_err(|e| e.to_string())?,
                free: *library.get(b"mpv_free\0").map_err(|e| e.to_string())?,
                wait: *library
                    .get(b"mpv_wait_event\0")
                    .map_err(|e| e.to_string())?,
                destroy: *library
                    .get(b"mpv_terminate_destroy\0")
                    .map_err(|e| e.to_string())?,
                error_string: *library
                    .get(b"mpv_error_string\0")
                    .map_err(|e| e.to_string())?,
                entry_id: None,
                _library: library,
            };
            mpv.handle = create();
            if mpv.handle.is_null() {
                return Err("Could not create the audio engine".into());
            }
            for (key, value) in [
                ("config", "no"),
                ("load-scripts", "no"),
                ("ytdl", "no"),
                ("vid", "no"),
                ("audio-display", "no"),
                ("terminal", "no"),
                ("input-default-bindings", "no"),
                // YouTube's CDN can refuse open-ended requests even for valid
                // signed URLs. The pinned libcurl backend supports bounded ranges.
                ("curl-max-request-size", "65536"),
                ("cache-secs", "10"),
                ("demuxer-max-bytes", "33554432"),
                ("demuxer-max-back-bytes", "8388608"),
                ("pause", "yes"),
            ] {
                let (key, value) = (CString::new(key).unwrap(), CString::new(value).unwrap());
                mpv.check(option(mpv.handle, key.as_ptr(), value.as_ptr()))?;
            }
            if null_audio {
                mpv.check(option(mpv.handle, c"ao".as_ptr(), c"null".as_ptr()))?;
            }
            mpv.check(initialize(mpv.handle))?;
            Ok(mpv)
        }
    }
    fn check(&self, code: i32) -> Result<(), String> {
        if code >= 0 {
            return Ok(());
        }
        // SAFETY: libmpv returns a static NUL-terminated error string.
        let message = unsafe { CStr::from_ptr((self.error_string)(code)) }
            .to_string_lossy()
            .into_owned();
        Err(message)
    }
    pub fn command(&self, args: &[&str]) -> Result<(), String> {
        let args = args
            .iter()
            .map(|s| CString::new(*s).map_err(|_| "Invalid NUL in media path/command"))
            .collect::<Result<Vec<_>, _>>()?;
        let mut pointers = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        pointers.push(std::ptr::null());
        // SAFETY: all arguments and the terminating null remain alive for the synchronous call.
        self.check(unsafe { (self.command)(self.handle, pointers.as_ptr()) })
    }
    pub fn flag(&self, key: &str, value: bool) -> Result<(), String> {
        let key = CString::new(key).unwrap();
        let mut value = i32::from(value);
        // SAFETY: MPV_FORMAT_FLAG (3) takes an int pointer; call does not retain it.
        self.check(unsafe {
            (self.set)(
                self.handle,
                key.as_ptr(),
                3,
                (&mut value as *mut i32).cast(),
            )
        })
    }
    pub fn set_number(&self, key: &str, mut value: f64) -> Result<(), String> {
        if !value.is_finite() {
            return Err("Invalid playback value".into());
        }
        let key = CString::new(key).unwrap();
        // SAFETY: MPV_FORMAT_DOUBLE (5) takes a double pointer.
        self.check(unsafe {
            (self.set)(
                self.handle,
                key.as_ptr(),
                5,
                (&mut value as *mut f64).cast(),
            )
        })
    }
    pub fn number(&self, key: &str) -> Option<f64> {
        let key = CString::new(key).ok()?;
        let mut value = 0f64;
        // SAFETY: matching double storage for MPV_FORMAT_DOUBLE.
        let result = unsafe {
            (self.get)(
                self.handle,
                key.as_ptr(),
                5,
                (&mut value as *mut f64).cast(),
            )
        };
        (result >= 0 && value.is_finite()).then_some(value)
    }
    pub fn string(&self, key: &str) -> Option<String> {
        let key = CString::new(key).ok()?;
        // SAFETY: returned allocation belongs to libmpv and is released exactly once with mpv_free.
        unsafe {
            let p = (self.get_string)(self.handle, key.as_ptr());
            if p.is_null() {
                return None;
            }
            let value = CStr::from_ptr(p).to_string_lossy().into_owned();
            (self.free)(p.cast());
            Some(value)
        }
    }
    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        self.load_media(path.to_str().ok_or("Invalid Unicode media path")?, &[])
    }
    pub fn load_media(&mut self, source: &str, headers: &[(String, String)]) -> Result<(), String> {
        self.command(&["change-list", "http-header-fields", "clr", ""])?;
        for (key, value) in headers {
            self.command(&[
                "change-list",
                "http-header-fields",
                "append",
                &format!("{key}: {value}"),
            ])?;
        }

        for _ in 0..512 {
            if unsafe { (*((self.wait)(self.handle, 0.0))).id } == 0 {
                break;
            }
        }
        self.entry_id = None;
        self.flag("pause", true)?;
        self.command(&["loadfile", source, "replace"])
    }
    pub fn poll(&mut self) -> Vec<Event> {
        let mut result = Vec::new();
        for _ in 0..32 {
            // SAFETY: event storage/data remain valid until the next mpv_wait_event call.
            unsafe {
                let event = &*(self.wait)(self.handle, 0.0);
                match event.id {
                    0 => break,
                    6 if !event.data.is_null() => {
                        self.entry_id = Some((*(event.data as *const StartFile)).entry_id);
                    }
                    8 if self.entry_id.is_some() => result.push(Event::Loaded),
                    7 if !event.data.is_null() => {
                        let end = &*(event.data as *const EndFile);
                        if self.entry_id == Some(end.entry_id) {
                            if end.reason == 0 {
                                result.push(Event::Ended);
                            }
                            if end.reason == 4 {
                                result.push(Event::Failed(
                                    self.check(end.error).err().unwrap_or_else(|| {
                                        "Unsupported or damaged audio file".into()
                                    }),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        result
    }
}
impl Drop for Mpv {
    fn drop(&mut self) {
        // SAFETY: this worker owns the handle exclusively; library is still loaded.
        if !self.handle.is_null() {
            unsafe {
                (self.destroy)(self.handle);
            }
        }
    }
}
