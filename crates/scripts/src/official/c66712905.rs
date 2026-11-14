// Runick Dispelling
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 66712905;

pub const LISTED_SERIES: &[u16] = &[SET_RUNICK];

struct Quickplay;

impl MysteruneQuickPlay for Quickplay {
    const ID: u32 = ID;
    const UNIQUECAT: u64 = CATEGORY_TOGRAVE + CATEGORY_HANDES;
    const RMCOUNT: usize = 2;
    const UNIQUEPROP: Option<u32> = Some(EFFECT_FLAG_DELAY);

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        tgtg(duel, a, chk)
    }

    fn uniqueop(duel: &mut Duel, a: &Args) -> Script<bool> {
        tgop(duel, a)
    }
}

// Copies in hand to activate when the opponent draws outside the Draw Phase, Hugin the Runick Wings to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (55990317, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(55990317)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let (e1, e2) = Mysterune::create_qp_effects::<Quickplay>(duel, c, EVENT_TO_HAND);
    e1.set_condition(duel, tgcon);
    c.register_effect(duel, e1);
    c.register_effect(duel, e2);
}

fn tgconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, 1 - tp) && c.is_previous_location(duel, LOCATION_DECK)
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.get_current_phase() != PHASE_DRAW
        && a.eg
            .is_some_and(|eg| eg.is_exists(duel, |c, duel| tgconfilter(c, duel, a.tp), 1, None))
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - tp, 1);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g.is_empty() {
        return Ok(false);
    }
    let sg = g.random_select(duel, 1 - tp, 1);
    Ok(!sg.is_empty() && duel.send_to_grave(sg, REASON_EFFECT | REASON_DISCARD)? > 0)
}
