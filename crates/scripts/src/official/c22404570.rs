// Power Patron Shade of the Final Hour
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 22404570;

// Artmage Power Patron in hand and the Graveyard to Special Summon, with Nerva (Fusion), Junora (Synchro) and Jupiter (Xyz) on the
// field for the three banishing options, and a monster and a Spell of the opponent to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (23829452, HAND),
    (23829452, GRAVE),
    (53589300, MZONE),
    (5914858, MZONE),
    (68231287, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_POWER_PATRON])
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_HAND | LOCATION_GRAVE) {
        duel.get_mzone_count(tp, None) > 0
    } else {
        c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_EXTRA | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_EXTRA | LOCATION_GRAVE) as i32,
    );
    duel.set_possible_operation_info(
        0,
        CATEGORY_REMOVE,
        None,
        1,
        1 - tp,
        (LOCATION_DECK | LOCATION_ONFIELD | LOCATION_HAND) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if g.is_empty() || duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let card_type_count = duel
        .get_matching_group(aux::faceup_filter(CardId::is_monster), tp, LOCATION_MZONE, 0, None)
        .get_bin_class_count(duel, |c, duel| {
            u64::from(c.get_type(duel) & (TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ))
        });
    if card_type_count == 0 {
        return Ok(());
    }
    let opp = 1 - tp;
    let mut break_chk = false;
    let opp_decktop3 = duel.get_decktop_group(opp, 3);
    let opp_field = duel.get_field_group(opp, LOCATION_ONFIELD, 0);
    let opp_hand = duel.get_field_group(opp, LOCATION_HAND, 0);
    let removable = |c: CardId, duel: &Duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT);
    if card_type_count >= 1
        && opp_decktop3.len() == 3
        && opp_decktop3.filter_count(duel, removable, None) == 3
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.disable_shuffle_check(true);
        duel.break_effect()?;
        if duel.remove(opp_decktop3, POS_FACEDOWN, REASON_EFFECT)? > 0 {
            break_chk = true;
        }
    }
    if card_type_count >= 2
        && !opp_field.is_empty()
        && opp_field.is_exists(duel, removable, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = opp_field.filter_select(duel, tp, removable, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            if break_chk {
                duel.break_effect()?;
            }
            if duel.remove(g, POS_FACEDOWN, REASON_EFFECT)? > 0 {
                break_chk = true;
            }
        }
    }
    if card_type_count == 3
        && !opp_hand.is_empty()
        && opp_hand.is_exists(duel, removable, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        let g = opp_hand.filter(duel, removable, None).random_select(duel, tp, 1);
        if !g.is_empty() {
            if break_chk {
                duel.break_effect()?;
            }
            duel.remove(g, POS_FACEDOWN, REASON_EFFECT)?;
        }
    }
    Ok(())
}
