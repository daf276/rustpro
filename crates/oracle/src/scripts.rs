use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::Path;

use ocgcore_sys::OCG_LoadScript;

// Every script preloaded, so duels never touch the filesystem and threads share one read-only copy.
#[derive(Debug, Clone, Default)]
pub struct Scripts {
    sources: HashMap<String, Vec<u8>>,
}

impl Scripts {
    // Same search as EDOPro's `Game::PopulateResourcesDirectories`: the root, then every subfolder; the first hit wins.
    pub fn load(root: &Path) -> std::io::Result<Self> {
        let mut directories = vec![root.to_path_buf()];
        for entry in std::fs::read_dir(root)? {
            let path = entry?.path();
            if path.is_dir() {
                directories.push(path);
            }
        }
        directories.sort();
        let mut sources = HashMap::new();
        for directory in directories {
            for entry in std::fs::read_dir(directory)? {
                let path = entry?.path();
                let Some(name) = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| name.ends_with(".lua"))
                else {
                    continue;
                };
                if let Entry::Vacant(slot) = sources.entry(name.to_owned()) {
                    slot.insert(std::fs::read(&path)?);
                }
            }
        }
        Ok(Self { sources })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.sources.contains_key(name)
    }

    pub(crate) fn load_into(&self, duel: ocgcore_sys::Duel, name: &CStr) -> bool {
        let Some(source) = name.to_str().ok().and_then(|name| self.sources.get(name)) else {
            return false;
        };
        let Ok(length) = u32::try_from(source.len()) else { return false };
        // SAFETY: duel is live for the duration of the callback or constructor calling this.
        unsafe { OCG_LoadScript(duel, source.as_ptr().cast(), length, name.as_ptr()) != 0 }
    }
}

pub(crate) extern "C" fn read_script(payload: *mut c_void, duel: ocgcore_sys::Duel, name: *const c_char) -> c_int {
    // SAFETY: payload is the Scripts borrowed by the duel, name is a NUL-terminated string from the core.
    let (scripts, name) = unsafe { (&*payload.cast::<Scripts>(), CStr::from_ptr(name)) };
    scripts.load_into(duel, name).into()
}
