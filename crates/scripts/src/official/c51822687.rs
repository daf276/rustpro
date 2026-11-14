// Expurrely Happiness
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 51822687;

pub const LISTED_SERIES: &[u16] = &[SET_PURRELY];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

// In the Extra Deck, Xyz Summoned with two Level 7 monsters (Dark Magician), with Sangan to negate. The damage effect needs five Xyz
// Materials, which no scenario attaches.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (36996508, MZONE),
    (36996508, MZONE),
    (36996508, MZONE),
    (6740720, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, dmgcon);
    e2.set_target(duel, dmgtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
}

fn ovfilter(c: CardId, duel: &Duel, _: u8, _: CardId) -> bool {
    c.is_faceup(duel) && c.is_rank(duel, &[2]) && c.get_overlay_count(duel) >= 5
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.get_original_level(duel) == 1
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT)
            && duel.is_existing_matching_card(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, None));
    }
    if c.get_overlay_group(duel).is_exists(duel, disfilter, 1, None) {
        duel.set_chain_limit(|_, _, ep, tp, _| tp == ep, 0);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, None);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0 && !g.is_empty() {
        for tc in g {
            tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
        }
    }
    Ok(())
}

fn dmgcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_faceup(duel) && c.is_relate_to_battle(duel) && c.get_overlay_count(duel) >= 5
}

fn dmgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_target_player(1 - tp);
    duel.set_target_param(1500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1500);
    Ok(false)
}
