// Knightmare Corruptor Iblee
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 10158145;

// Knightmare Link Monsters in the GY to summon, copies in hand to Normal Summon, Dark Hole to send it from the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (2857636, GRAVE),
    (75452921, GRAVE),
    (38342335, GRAVE),
    (ID, HAND),
    (ID, HAND),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, splimit);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, condition);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, CARD_EHERO_BLAZEMAN);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 1, 0);
    c.register_effect(duel, e4);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ec: CardId) -> bool {
    let zone = ec.get_to_be_linked_zone(duel, c, tp);
    zone != 0 && c.is_link_monster(duel) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ec = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp, ec));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp, ec), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp, ec), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) {
        let zone = c.get_to_be_linked_zone(duel, tc, tp);
        if zone != 0 && duel.special_summon_step_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)? {
            for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT, EFFECT_SET_ATTACK_FINAL] {
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, code);
                if code == EFFECT_SET_ATTACK_FINAL {
                    e1.set_value(duel, 0);
                }
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e1);
            }
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    if c.is_monster(duel) {
        !c.is_type(duel, TYPE_LINK)
    } else {
        !c.is_original_type(duel, TYPE_LINK)
    }
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_previous_controler(duel, a.tp)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, a.e, 0, tp, false, false, POS_FACEUP_DEFENSE, 1 - tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, 1 - a.tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
