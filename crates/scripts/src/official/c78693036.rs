// Juraishin, the Cursed Thunder God
use engine::constants::MSG_RECOVER;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 78693036;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned from Deep Sea Diva and Veidos, and on the field, with the opponent's monster and
// Raigeki to activate effects and have its destruction effect used.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (78868119, MZONE),
    (78783557, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Message(MSG_RECOVER),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1a.set_code(duel, EVENT_CHAINING);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1b.set_property(duel, EFFECT_FLAG_DELAY);
    e1b.set_code(duel, EVENT_CHAIN_SOLVED);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, 1));
    e1b.set_operation(duel, lpop);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::pay_lp::<1500>, Cost::self_tribute])
    });
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if a.rp == 1 - tp {
        duel.hint(HINT_CARD, 0, u64::from(ID));
        duel.recover(tp, 300, REASON_EFFECT)?;
    }
    Ok(())
}

fn faceup_spell_trap() -> impl Fn(CardId, &Duel) -> bool {
    aux::faceup_filter(CardId::is_spell_trap)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g1 = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    let equips = e.get_handler(duel).get_equip_group(duel);
    let g2 = duel.get_matching_group(faceup_spell_trap(), tp, 0, LOCATION_ONFIELD, Some(equips));
    let (b1, b2) = (!g1.is_empty(), !g2.is_empty());
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    let g = if op == Some(1) { g1 } else { g2 };
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = match a.e.get_label(duel) {
        1 => duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None),
        2 => duel.get_matching_group(faceup_spell_trap(), tp, 0, LOCATION_ONFIELD, None),
        _ => return Ok(()),
    };
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
