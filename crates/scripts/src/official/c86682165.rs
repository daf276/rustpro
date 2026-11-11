// Icejade Gymir Aegirine
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 86682165;

// In the Extra Deck, Synchro Summoned from Guardragon Justicia, Mystical Elf and Flying Kamakiri #2, a second copy in the
// Graveyard to be banished by the opponent's Called by the Grave and Special Summoned back.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (11012154, MZONE),
    (15025844, MZONE),
    (3134241, MZONE),
    (24224830, HAND | OPPONENT),
    (24224830, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_target(duel, immtg);
    e1.set_operation(duel, immop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None))
    });
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn rmfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_faceup(duel) && c.is_code(duel, code) && c.is_able_to_remove(duel)
}

fn immtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - a.tp, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn immop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_REMOVE);
    e2.set_target_range(duel, 1, 1);
    e2.set_player_card_target(duel, remtg);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    aux::register_client_hint_full(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)), RESET_PHASE | PHASE_END, 0);
    let ch = duel.get_current_chain();
    if ch <= 1 {
        return Ok(());
    }
    let chp: u8 = duel.get_chain_info(ch - 1, CHAININFO_TRIGGERING_PLAYER);
    let chcode: u32 = duel.get_chain_info(ch - 1, CHAININFO_TRIGGERING_CODE);
    if chp != 1 - tp {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| rmfilter(c, duel, chcode), tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.break_effect()?;
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn remtg(duel: &Duel, e: EffectId, c: CardId, rp: u8, r: u32) -> bool {
    let tp = e.get_handler_player(duel);
    c.is_faceup(duel) && c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE) && rp == 1 - tp && r & REASON_EFFECT != 0
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_EFFECT) && c.get_reason_player(duel) == 1 - tp
}
