// Zerrziel, Ruler of the Evil Eyed
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 17739335;

pub const LISTED_SERIES: &[u16] = &[SET_EVIL_EYE];

const CATOBLEPAS: u32 = 45955628;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_EVIL_EYE], Some(lc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_EVIL_EYE_SELENE];

// Link Summoned from Catoblepas, or on the field with Evil Eye of Selene to equip and monsters of the opponent to target and negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CATOBLEPAS, MZONE),
    (CATOBLEPAS, MZONE),
    (CATOBLEPAS, MZONE),
    (CATOBLEPAS, GRAVE),
    (CARD_EVIL_EYE_SELENE, HAND),
    (CARD_EVIL_EYE_SELENE, DECK),
    (14558127, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, descond);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, discond);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn matcheck(duel: &mut Duel, e: EffectId, _: CardId) {
    let c = e.get_handler(duel);
    if c.get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_attack_above(duel, 2600), 1, None)
    {
        let reset = RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_reset(duel, reset);
        c.register_effect(duel, e1);
        c.register_flag_effect_full(duel, 0, reset, EFFECT_FLAG_CLIENT_HINT, 1, 0, 3201);
    }
}

fn descond(duel: &Duel, a: &Args) -> bool {
    let eg = a.e.get_handler(duel).get_equip_group(duel);
    !eg.is_empty() && eg.is_exists(duel, |c, duel| c.is_code(duel, CARD_EVIL_EYE_SELENE), 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    let reset = RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY;
    let c = e.get_handler(duel);
    if duel.is_phase(PHASE_STANDBY) {
        c.register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_OATH, 2, i64::from(duel.get_turn_count()), 0);
    } else {
        c.register_flag_effect(duel, ID, reset, EFFECT_FLAG_OATH, 1);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn discond(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_flag_effect_label(duel, ID)
        .is_some_and(|label| label != i64::from(duel.get_turn_count()))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(lg), 1, 0, 0);
    Ok(false)
}

fn ngtfilt(c: CardId, duel: &Duel, e: EffectId) -> bool {
    let lg = e.get_handler(duel).get_linked_group(duel);
    c.is_type(duel, TYPE_EFFECT) && c.is_faceup(duel) && !c.is_disabled(duel) && lg.is_contains(c)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
    let g = duel.select_matching_card(tp, |c, duel| ngtfilt(c, duel, e), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
