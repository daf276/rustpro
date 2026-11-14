// D/D Gryphon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, PZONE, SZONE};

const ID: u32 = 28406301;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_DARK_CONTRACT, SET_DD];
// In a Pendulum Zone and the hand, with face-up D/D Fiends on the field, Dark Contract with the Witch on the field, and
// D/D cards in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (72181263, MZONE),
    (72181263, MZONE),
    (67322708, MZONE),
    (33334269, MZONE),
    (9765723, SZONE),
    (9765723, SZONE),
    (33334269, DECK),
    (67322708, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DD]));
        duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_pendulum_summoned(duel));
    e3.set_cost(duel, |duel, a, chk| {
        Cost::discard_full(
            duel,
            a,
            chk,
            |c, duel| c.is_set_card(duel, &[SET_DD, SET_DARK_CONTRACT]),
            false,
            (1, 1),
        )
    });
    e3.set_target(duel, stock::draw_target::<1>);
    e3.set_operation(duel, stock::draw);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_count_limit(duel, 1, (ID, 3));
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_GRAVE));
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
}

fn atkctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_CONTRACT]) && c.is_spell_trap(duel) && c.is_faceup(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, tp)
            && chkc.is_faceup(duel)
            && chkc.is_race(duel, RACE_FIEND));
    }
    let fiend = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_FIEND));
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(atkctfilter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
                && duel.is_existing_target(fiend, tp, LOCATION_MZONE, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    duel.select_target(tp, fiend, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && !tc.is_immune_to_effect(duel, e) {
        let g = duel.get_matching_group(atkctfilter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, None);
        if g.is_empty() {
            return Ok(());
        }
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32 * 500);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        if c.is_relate_to_effect(duel, e) {
            duel.break_effect()?;
            duel.destroy(c, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DD]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
