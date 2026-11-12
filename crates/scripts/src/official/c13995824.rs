// Voltic Bicorn
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 13995824;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_BEAST, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Key Mouse (Level 1 Beast Tuner) and Frostosaurus (Level 6), then destroyed by the
// opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (135598, MZONE),
    (135598, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (12580477, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_condition(duel, ddcon);
    e1.set_target(duel, ddtg);
    e1.set_operation(duel, ddop);
    c.register_effect(duel, e1);
}

fn ddcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
}

fn ddtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, PLAYER_ALL, 7);
    Ok(false)
}

fn ddop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(a.tp, 7, REASON_EFFECT)?;
    duel.discard_deck(1 - a.tp, 7, REASON_EFFECT)?;
    Ok(())
}
