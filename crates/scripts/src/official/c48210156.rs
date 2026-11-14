// D/D Nighthowl
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 48210156;

pub const LISTED_SERIES: &[u16] = &[SET_DD];
// Normal Summoned with D/D Pandora in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (32146097, GRAVE), (32146097, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DD]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        // The Lua compares the step's boolean with 0, which is always true: the effects register even if it failed.
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
        tc.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e3.set_code(duel, EVENT_DESTROYED);
        e3.set_condition(duel, damcon);
        e3.set_operation(duel, damop);
        duel.register_effect(e3, tp);
        let e4 = duel.create_effect(c);
        e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e4.set_code(duel, EVENT_DESTROY);
        e4.set_label_object(duel, Some(e3));
        e4.set_operation(duel, checkop);
        e4.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e4);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.get_race(duel) != RACE_FIEND);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    duel.special_summon_complete()?;
    Ok(())
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(e3) = a.e.get_label_object(duel) {
        e3.set_label(duel, 1);
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label(duel) == 1
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.damage(a.tp, 1000, REASON_EFFECT)?;
    a.e.set_label(duel, 0);
    a.e.reset(duel);
    Ok(())
}
