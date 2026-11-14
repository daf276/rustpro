// Branded Lost
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 18973184;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];
// On the field with Polymerization, Fallen of Albaz and Aluber the Jester of Despia to Fusion Summon Despian Quaeritis; Fallen of
// Albaz in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (CARD_POLYMERIZATION, HAND),
    (CARD_ALBAZ, MZONE),
    (62962630, MZONE),
    (62962630, HAND),
    (72272462, EXTRA),
    (72272462, EXTRA),
    (CARD_ALBAZ, DECK),
    (CARD_ALBAZ, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_INACTIVATE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_value(duel, Value::Chain(efilter));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| limfilter(c, duel, a.tp), 1, None)
    });
    e3.set_operation(duel, limop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_code(duel, EVENT_CHAIN_END);
    e4.set_operation(duel, limop2);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_count_limit(duel, 1, (ID, 0));
    e5.set_condition(duel, |duel, a| {
        a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None)
    });
    e5.set_target(duel, |duel, a, chk, _| {
        stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
    });
    e5.set_operation(duel, thop);
    c.register_effect(duel, e5);
}

fn efilter(duel: &Duel, e: EffectId, ct: u8) -> bool {
    let tp = e.get_handler_player(duel);
    let te: Option<EffectId> = duel.get_chain_info(ct, CHAININFO_TRIGGERING_EFFECT);
    let rp: u8 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_PLAYER);
    tp == rp && te.is_some_and(|te| te.is_has_category(duel, CATEGORY_FUSION_SUMMON))
}

fn limfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_fusion_summoned(duel)
}

fn chainlm(_: &Duel, _: EffectId, rp: u8, tp: u8, _: i64) -> bool {
    tp == rp
}

fn limop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    match duel.get_current_chain() {
        0 => duel.set_chain_limit_till_chain_end(chainlm, 0),
        1 => {
            c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e1.set_code(duel, EVENT_CHAINING);
            e1.set_operation(duel, resetop);
            duel.register_effect(e1, a.tp);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EVENT_BREAK_EFFECT);
            e2.set_reset(duel, RESET_CHAIN);
            duel.register_effect(e2, a.tp);
        }
        _ => {}
    }
    Ok(())
}

fn resetop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).reset_flag_effect(duel, ID);
    a.e.reset(duel);
    Ok(())
}

fn limop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.get_flag_effect(duel, ID) > 0 {
        duel.set_chain_limit_till_chain_end(chainlm, 0);
    }
    c.reset_flag_effect(duel, ID);
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    limfilter(c, duel, tp) && c.is_type(duel, TYPE_FUSION)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_ALBAZ) || (c.is_monster(duel) && c.lists_code(duel, CARD_ALBAZ))) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
