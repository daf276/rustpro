// R.B. Operation Test
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 16066654;

pub const LISTED_SERIES: &[u16] = &[SET_RB];
// R.B. Lambda Cannon in the Graveyard to revive and in hand to summon, R.B. Link Monsters face-up to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (81794107, GRAVE),
    (81794107, GRAVE),
    (81794107, HAND),
    (6821579, MZONE),
    (80071619, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(81794107),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(
        duel,
        CATEGORY_RECOVER + CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON,
    );
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, recthsptg);
    e2.set_operation(duel, recthspop);
    c.register_effect(duel, e2);
}

fn spfromgyfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 3) && c.is_set_card(duel, &[SET_RB]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfromgyfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    let filter = |c, duel: &Duel| spfromgyfilter(c, duel, e, tp);
    if duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    } else {
        e.set_category(duel, 0);
        e.set_property(duel, 0);
    }
    Ok(false)
}

fn recthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RB]) && c.is_faceup(duel) && (c.is_able_to_hand(duel) || c.is_able_to_extra(duel))
}

fn recthsptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && recthfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(recthfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    let ct = duel.get_target_count(recthfilter, tp, LOCATION_MZONE, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel
        .select_target(tp, recthfilter, tp, LOCATION_MZONE, 0, 1, ct as u8, None)?
        .unwrap_or_default();
    let sum = g.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel)));
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, sum as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfromhandfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RB]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn recthspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if tg.is_empty() {
        return Ok(());
    }
    let sum = tg.get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel)));
    let filter = |c, duel: &Duel| spfromhandfilter(c, duel, e, tp);
    if duel.recover(tp, sum as u32, REASON_EFFECT)? > 0
        && duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)? > 0
        && duel.get_mzone_count(tp, None) > 0
        && tg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND | LOCATION_EXTRA), None) > 0
        && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None)
    {
        duel.shuffle_hand(tp)?;
        if !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
