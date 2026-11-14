// Lunalight Scarlet Tiger
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 11876803;

pub const LISTED_SERIES: &[u16] = &[SET_LUNALIGHT];

const EXTRA_EFFECT: u8 = 0;
const LUNALIGHT_KALEIDO_CHICK: u32 = 35618217;
const DARK_HOLE: u32 = 53129443;

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];

// In hand with Lunalight Kaleido Chick on the field to activate, Polymerization in the Deck, and the opponent's Dark Hole
// and Elfnote Lucina to destroy a copy on the field and chain to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (LUNALIGHT_KALEIDO_CHICK, MZONE),
    (LUNALIGHT_KALEIDO_CHICK, GRAVE),
    (CARD_POLYMERIZATION, DECK),
    (DARK_HOLE, HAND | OPPONENT),
    (13597785, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| {
        a.rp == a.tp
            && a.re
                .is_some_and(|re| re.is_card_setcode(duel, SET_LUNALIGHT) && re.is_monster_effect(duel))
    });
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(|_, _, _, _| false));
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e2.set_target(duel, exgysptg);
    e2.set_operation(duel, exgyspop);
    c.register_effect(duel, e2);
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let extra_effect = duel.get_custom_activity_count(ID, 1 - tp, ACTIVITY_CHAIN) > 0;
    e.set_chain_data(duel, EXTRA_EFFECT, i64::from(extra_effect));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && e.get_chain_data(duel, EXTRA_EFFECT).is_some_and(|extra_effect| extra_effect != 0)
    {
        let filter = aux::necro_valley_filter(thfilter);
        let g = duel.get_matching_group(&filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
                if sc.is_location(duel, LOCATION_GRAVE) {
                    duel.hint_selection(sc);
                }
                duel.break_effect()?;
                duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
                if sc.is_previous_location(duel, LOCATION_DECK) {
                    duel.confirm_cards(1 - tp, sc)?;
                }
            }
        }
    }
    Ok(())
}

fn exgyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    if !(c.is_set_card(duel, &[SET_LUNALIGHT])
        && !c.is_level(duel, &[5])
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        c.is_location(duel, LOCATION_GRAVE) && mmz_chk
    }
}

fn exgysptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_EXTRA | LOCATION_GRAVE;
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| exgyspfilter(c, duel, e, tp, mmz_chk), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn exgyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| exgyspfilter(c, duel, e, tp, mmz_chk));
    let sc = duel
        .select_matching_card(tp, filter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3302);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}
