use std::ffi::c_void;

use carddata::{Card, CardDatabase};
use ocgcore_sys::CardData;

fn to_ffi(card: &Card) -> CardData {
    CardData {
        code: card.code,
        alias: card.alias,
        setcodes: card.setcodes.as_ptr().cast_mut(),
        card_type: card.card_type,
        level: card.level,
        attribute: card.attribute,
        race: card.race,
        attack: card.attack,
        defense: card.defense,
        lscale: card.lscale,
        rscale: card.rscale,
        link_marker: card.link_marker,
    }
}

// The core copies setcodes after this returns, so they point into the database rather than a temporary.
pub(crate) extern "C" fn read_card(payload: *mut c_void, code: u32, data: *mut CardData) {
    // SAFETY: payload is the CardDatabase borrowed by the duel, data is a zeroed struct owned by the core.
    let (database, data) = unsafe { (&*payload.cast::<CardDatabase>(), &mut *data) };
    if let Some(card) = database.get(code) {
        *data = to_ffi(card);
    }
}
