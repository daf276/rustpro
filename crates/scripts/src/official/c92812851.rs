// Exceed the Pendulum
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 92812851;
const JUNOLDO: u32 = 10266279;
const NERVEDO: u32 = 17473466;
const JUPREDO: u32 = 43871165;
const FLASH_KNIGHT: u32 = 17390179;
const VIDOLIUM: u32 = 70488851;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        g.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_PENDULUM, Some(lc), sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned from Nervedo and Sangan with Nervedo face-up in the Extra Deck to search, and on the field with
// Junoldo and Vidolium as scales and Flash Knight and Jupredo in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (NERVEDO, MZONE),
    (26202165, MZONE),
    (NERVEDO, EXTRA),
    (JUNOLDO, LOCATION_PZONE as u16),
    (VIDOLIUM, LOCATION_PZONE as u16),
    (FLASH_KNIGHT, HAND),
    (JUPREDO, HAND),
    (FLASH_KNIGHT, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| {
            duel.get_matching_group_count(pendfilter, e.get_handler_player(duel), LOCATION_ONFIELD, 0, None) as i32 * 100
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        duel.is_main_phase()
            && duel.get_field_card(a.tp, LOCATION_PZONE, 0).is_some()
            && duel.get_field_card(a.tp, LOCATION_PZONE, 1).is_some()
    });
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn pendfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_original_type(duel, TYPE_PENDULUM) && c.is_original_type(duel, TYPE_MONSTER)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lsc: u32, rsc: u32) -> bool {
    let lv = c.get_level(duel);
    c.is_type(duel, TYPE_PENDULUM)
        && c.has_level(duel)
        && lv > lsc
        && lv < rsc
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn scales(duel: &Duel, tp: u8) -> Option<(u32, u32)> {
    let lsc = duel.get_field_card(tp, LOCATION_PZONE, 0)?.get_left_scale(duel);
    let rsc = duel.get_field_card(tp, LOCATION_PZONE, 1)?.get_right_scale(duel);
    Some((lsc.min(rsc), lsc.max(rsc)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some((lsc, rsc)) = scales(duel, tp) else { return Ok(false) };
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp, lsc, rsc),
                tp,
                LOCATION_GRAVE | LOCATION_HAND,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_GRAVE | LOCATION_HAND) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let Some((lsc, rsc)) = scales(duel, tp) else { return Ok(()) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, lsc, rsc));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
