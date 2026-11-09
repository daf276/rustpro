// Visas Amritara
use super::c65961304::valcheck;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 821049;

pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST];

// In the Extra Deck, Synchro Summoned with Angel Trumpeter and Mystical Elf, with Mannadium Reframing in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (15025844, MZONE),
    (18158393, DECK),
    (18158393, DECK),
    (18158393, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MAX1: usize = 99;
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, CARD_VISAS_STARFROST as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_MATERIAL_CHECK);
    e4.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_VISAS_STARFROST) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_field_group(a.tp, LOCATION_MZONE, 0);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_SYNCHRO));
    e1.set_value(duel, 800);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    Ok(())
}
