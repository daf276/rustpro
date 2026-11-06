use std::ffi::{CStr, c_char, c_int, c_void};
use std::marker::PhantomData;
use std::ptr;

use carddata::{CardDatabase, Deck};
use ocgcore_sys as ffi;

use crate::cards::read_card;
use crate::scripts::{Scripts, read_script};

#[derive(Debug, thiserror::Error)]
pub enum DuelError {
    #[error("duel creation failed with status {0}")]
    Creation(c_int),
    #[error("startup script {0:?} failed to load")]
    StartupScript(&'static CStr),
    #[error("unknown duel status {0}")]
    UnknownStatus(c_int),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    End,
    Awaiting,
    Continue,
}

#[derive(Debug, Default)]
struct ErrorLog {
    errors: Vec<String>,
    traceback: Option<String>,
}

pub struct Duel<'a> {
    handle: ffi::Duel,
    errors: Box<ErrorLog>,
    sources: PhantomData<(&'a CardDatabase, &'a Scripts)>,
}

impl<'a> Duel<'a> {
    pub fn new(cards: &'a CardDatabase, scripts: &'a Scripts, seed: [u64; 4]) -> Result<Self, DuelError> {
        let player = ffi::Player {
            starting_lp: 8000,
            starting_draw_count: 5,
            draw_count_per_turn: 1,
        };
        let mut duel = Self {
            handle: ptr::null_mut(),
            errors: Box::default(),
            sources: PhantomData,
        };
        let options = ffi::DuelOptions {
            seed,
            flags: ffi::DUEL_MODE_MR5,
            team1: player,
            team2: player,
            card_reader: Some(read_card),
            card_reader_payload: ptr::from_ref(cards).cast_mut().cast(),
            script_reader: Some(read_script),
            script_reader_payload: ptr::from_ref(scripts).cast_mut().cast(),
            log_handler: Some(log_error),
            log_handler_payload: ptr::from_mut(duel.errors.as_mut()).cast(),
            card_reader_done: None,
            card_reader_done_payload: ptr::null_mut(),
            enable_unsafe_libraries: 0,
        };
        // SAFETY: options only points at data that outlives the duel; the core copies the struct itself.
        let status = unsafe { ffi::OCG_CreateDuel(&mut duel.handle, &options) };
        if status != ffi::DUEL_CREATION_SUCCESS {
            return Err(DuelError::Creation(status));
        }
        // The core never requests these itself; utility.lua pulls in the proc_* libraries.
        for name in [c"constant.lua", c"utility.lua"] {
            if !scripts.load_into(duel.handle, name) {
                return Err(DuelError::StartupScript(name));
            }
        }
        Ok(duel)
    }

    pub fn add_card(&mut self, info: &ffi::NewCardInfo) {
        // SAFETY: handle is a live duel.
        unsafe { ffi::OCG_DuelNewCard(self.handle, info) }
    }

    // Cards are added on top of the deck, so the last main deck card is drawn first.
    pub fn add_deck(&mut self, team: u8, deck: &Deck) {
        let main = deck.main.iter().map(|&code| (code, ffi::LOCATION_DECK));
        for (code, loc) in main.chain(deck.extra.iter().map(|&code| (code, ffi::LOCATION_EXTRA))) {
            self.add_card(&ffi::NewCardInfo {
                team,
                duelist: 0,
                code,
                con: team,
                loc,
                seq: 0,
                pos: ffi::POS_FACEDOWN_DEFENSE,
            });
        }
    }

    // Runs Lua in the duel, as a puzzle's Debug calls before the duel starts.
    pub fn run_lua(&mut self, source: &str) -> Result<(), DuelError> {
        let name = c"scenario.lua";
        let length = u32::try_from(source.len()).map_err(|_| DuelError::StartupScript(name))?;
        // SAFETY: handle is a live duel; the core copies the buffer.
        match unsafe { ffi::OCG_LoadScript(self.handle, source.as_ptr().cast(), length, name.as_ptr()) } {
            0 => Err(DuelError::StartupScript(name)),
            _ => Ok(()),
        }
    }

    pub fn start(&mut self) {
        // SAFETY: handle is a live duel.
        unsafe { ffi::OCG_StartDuel(self.handle) }
    }

    pub fn process(&mut self) -> Result<Status, DuelError> {
        // SAFETY: handle is a live duel.
        match unsafe { ffi::OCG_DuelProcess(self.handle) } {
            ffi::DUEL_STATUS_END => Ok(Status::End),
            ffi::DUEL_STATUS_AWAITING => Ok(Status::Awaiting),
            ffi::DUEL_STATUS_CONTINUE => Ok(Status::Continue),
            other => Err(DuelError::UnknownStatus(other)),
        }
    }

    // Each message is prefixed by its u32 length in native byte order.
    pub fn messages(&mut self) -> impl Iterator<Item = &[u8]> {
        let mut length = 0;
        // SAFETY: the core owns the buffer until the next call on this duel, which needs &mut self.
        let mut remaining = unsafe {
            let buffer = ffi::OCG_DuelGetMessage(self.handle, &mut length);
            if buffer.is_null() {
                &[][..]
            } else {
                std::slice::from_raw_parts(buffer.cast::<u8>(), length as usize)
            }
        };
        std::iter::from_fn(move || {
            let (size, rest) = remaining.split_first_chunk::<4>()?;
            let (message, rest) = rest.split_at_checked(u32::from_ne_bytes(*size) as usize)?;
            remaining = rest;
            Some(message)
        })
    }

    pub fn set_response(&mut self, response: &[u8]) {
        // SAFETY: handle is a live duel; the core copies the buffer.
        unsafe { ffi::OCG_DuelSetResponse(self.handle, response.as_ptr().cast(), response.len() as u32) }
    }

    // Each error is followed by the Lua traceback the core logged just before it, if any.
    pub fn take_errors(&mut self) -> Vec<String> {
        std::mem::take(&mut self.errors.errors)
    }
}

impl Drop for Duel<'_> {
    fn drop(&mut self) {
        // SAFETY: handle is null or a live duel, and OCG_DestroyDuel accepts both.
        unsafe { ffi::OCG_DestroyDuel(self.handle) }
    }
}

extern "C" fn log_error(payload: *mut c_void, message: *const c_char, log_type: c_int) {
    // SAFETY: payload is the duel's boxed ErrorLog, message is a NUL-terminated string from the core.
    let (log, message) = unsafe { (&mut *payload.cast::<ErrorLog>(), CStr::from_ptr(message).to_string_lossy()) };
    match log_type {
        ffi::LOG_TYPE_FOR_DEBUG => log.traceback = Some(message.into_owned()),
        ffi::LOG_TYPE_ERROR => {
            let traceback = log.traceback.take().map(|traceback| format!("\n{traceback}")).unwrap_or_default();
            log.errors.push(format!("{message}{traceback}"));
        }
        _ => {}
    }
}
