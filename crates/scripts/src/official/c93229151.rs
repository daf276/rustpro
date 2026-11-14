// Runick Smiting Storm
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};
use engine::constants::MSG_ANNOUNCE_NUMBER;

const ID: u32 = 93229151;

pub const LISTED_SERIES: &[u16] = &[SET_RUNICK];

struct Quickplay;

impl MysteruneQuickPlay for Quickplay {
    const ID: u32 = ID;
    const UNIQUECAT: u64 = 0;
    const RMCOUNT: usize = 1;

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        rmtg(duel, a, chk)
    }

    // The Lua's rmop returns nothing, so the library never banishes its own card afterwards.
    fn uniqueop(duel: &mut Duel, a: &Args) -> Script<bool> {
        rmop(duel, a)?;
        Ok(false)
    }
}

// Hugin the Runick Wings in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (55990317, EXTRA)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ANNOUNCE_NUMBER), Check::SpecialSummonedCode(55990317)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Mysterune::create_qp_effect::<Quickplay>(duel, c);
    c.register_effect(duel, e1);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, 0, LOCATION_ONFIELD) > 0);
    }
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let max = duel
        .get_field_group_count(tp, 0, LOCATION_DECK)
        .min(duel.get_field_group_count(tp, 0, LOCATION_ONFIELD));
    if max < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let ct = duel.announce_number_range(tp, 1, max as u32)?;
    if ct < 1 {
        return Ok(());
    }
    let rg = duel.get_decktop_group(1 - tp, ct as usize);
    if !rg.is_empty() {
        duel.disable_shuffle_check(true);
        duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
