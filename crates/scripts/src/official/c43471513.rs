// Convex Knight
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 43471513;

// In the hand and on the field, with Giant Mech-Soldier and Cycroid as the Machines on the field and Steel Ogre Grotto #1 in
// the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (72299832, MZONE),
    (45945685, MZONE),
    (29172562, DECK),
    (29172562, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_MACHINE));
        duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_LVCHANGE, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn lvfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    ((c.has_level(duel) && !c.is_level(duel, &[lv])) || (c.has_rank(duel) && !c.is_rank(duel, &[lv])))
        && c.is_race(duel, RACE_MACHINE)
        && c.is_faceup(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let lv = c.get_level(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0
        && duel.is_existing_matching_card(|c, duel| lvfilter(c, duel, lv), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        let Some(sc) = duel
            .select_matching_card(tp, |c, duel| lvfilter(c, duel, lv), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.hint_selection(sc);
        let lv_rnk = if sc.has_level(duel) {
            sc.get_level(duel)
        } else {
            sc.get_rank(duel)
        };
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv_rnk as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_race(duel, RACE_MACHINE) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    // The script's `c` is an undefined global here, so the group is nil.
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, None, 1, tp, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(sc) = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
    {
        duel.break_effect()?;
        let amt = sc.get_level(duel) as i32 * 100;
        c.update_attack(duel, amt, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None);
    }
    Ok(())
}
