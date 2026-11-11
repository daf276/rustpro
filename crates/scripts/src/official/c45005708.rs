// The Bystial Aluber
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 45005708;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

// In the hand to Normal Summon and discard, with Baby Dragon on the opponent's field to take and Petit Dragon in their Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (88819587, MZONE | OPPONENT),
    (88819587, MZONE | OPPONENT),
    (75356564, GRAVE | OPPONENT),
    (75356564, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_ALBAZ as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sc: CardId) -> bool {
    if !(c.is_race(duel, RACE_DRAGON) && c.is_faceup(duel)) {
        return false;
    }
    if c.is_location(duel, LOCATION_MZONE) {
        ctrlfilter(c, duel, tp, sc)
    } else {
        spfilter(c, duel, e, tp, sc)
    }
}

fn ctrlfilter(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    duel.get_mzone_count_full(tp, Some(sc.into()), tp, LOCATION_REASON_CONTROL, 0xff) > 0 && c.is_able_to_change_controler(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sc: CardId) -> bool {
    duel.get_mzone_count(tp, Some(sc.into())) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let label = e.get_label(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE)
            && ((label == 1 && ctrlfilter(chkc, duel, tp, c)) || (label == 2 && spfilter(chkc, duel, e, tp, c))));
    }
    if chk == 0 {
        return Ok(c.is_able_to_grave(duel)
            && duel.is_existing_target(
                |tc, duel| tgfilter(tc, duel, e, tp, c),
                tp,
                0,
                LOCATION_MZONE | LOCATION_GRAVE,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(
        tp,
        |tc, duel| tgfilter(tc, duel, e, tp, c),
        tp,
        0,
        LOCATION_MZONE | LOCATION_GRAVE,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    if tc.is_location(duel, LOCATION_MZONE) {
        e.set_label(duel, 1);
        e.set_category(duel, CATEGORY_TOGRAVE | CATEGORY_CONTROL);
        duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    } else {
        e.set_label(duel, 2);
        e.set_category(duel, CATEGORY_TOGRAVE | CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.send_to_grave(c, REASON_EFFECT)? > 0 && c.is_location(duel, LOCATION_GRAVE) {
        let Some(tc) = duel.get_first_target() else { return Ok(()) };
        if !tc.is_relate_to_effect(duel, e) {
            return Ok(());
        }
        let label = e.get_label(duel);
        if label == 1 {
            duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
        } else if label == 2 && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
