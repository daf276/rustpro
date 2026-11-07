// Wonder Xyz
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE};

const ID: u32 = 73860462;

// Two Level 4 Normal Monsters (Mechanicalchaser, Gradius) for Tornado Dragon in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (6983839, EXTRA), (7359741, MZONE), (10992251, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_target(duel, xyztg);
    e1.set_operation(duel, xyzop);
    c.register_effect(duel, e1);
}

pub(crate) fn xyz_summonable(c: CardId, duel: &Duel) -> bool {
    c.is_xyz_summonable(duel, None, None)
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(xyz_summonable, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(xyz_summonable, tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let tg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = tg.get_first() {
            duel.xyz_summon(tp, sc, None, None)?;
        }
    }
    Ok(())
}
