// Exosister Asophiel
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 41524885;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with Exosisters, with Sangan to bounce.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16474916, MZONE),
    (16474916, MZONE),
    (37343995, MZONE),
    (43863925, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_value(duel, Value::EffectPlayer(indvalue));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, limcon);
    e2.set_operation(duel, limop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
}

pub fn indvalue(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.is_activated(duel)
        && re.is_monster_effect(duel)
        && re.is_card_summon_type(duel, SUMMON_TYPE_SPECIAL)
        && re.is_card_summon_location(duel, LOCATION_GRAVE)
}

fn valcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c.get_material(duel);
    if g.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_EXOSISTER]), 1, None) {
        c.register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TOFIELD), 0, 1);
    }
}

fn limcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_xyz_summoned(duel) && c.get_flag_effect(duel, ID) > 0
}

fn limop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_activate_location(duel) == LOCATION_GRAVE),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
