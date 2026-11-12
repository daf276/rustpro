// Soul of Silvermountain
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 91895091;

// On the field with materials or Xyz Summoned with two Earth Level 3 Normal Monsters (Burglar), with set Traps (Trap Hole) to lock and Burglar in
// the GY to revive after battling Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (6297941, MZONE),
    (6297941, GRAVE),
    (6297941, GRAVE),
    (4206964, SZONE | OPPONENT),
    (4206964, SZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.get_sequence(duel) != 5
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_SZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    duel.select_target(tp, filter, tp, 0, LOCATION_SZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e)
        && let Some(tc) = tc
        && tc.is_facedown(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        c.set_card_target(duel, tc);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_OWNER_RELATE);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_condition(duel, rcon);
        e1.set_value(duel, 1);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn rcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_owner(duel).is_has_card_target(duel, a.e.get_handler(duel))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_relate_to_battle(duel) && c.get_overlay_count(duel) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
