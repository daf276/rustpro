// White Aura Porpoise
use super::c63731062::{Procedure, selfspcost, selfspop, selfsptg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 46956301;

// On the field, Synchro Summoned in the Extra Deck with Guardragon Justicia (a WATER Tuner) and Sky Scout (a Level 4
// Normal Monster), with Great White (a Level 4 Fish Normal Monster) in the hand and the Graveyard, and Raigeki in the
// opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (11012154, MZONE),
    (30532390, MZONE),
    (30532390, MZONE),
    (13429800, HAND),
    (13429800, GRAVE),
    (13429800, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, fishsptg);
    e1.set_operation(duel, fishspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, selfspcon);
    e2.set_cost(duel, selfspcost);
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
}

fn fishspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_race(duel, RACE_FISH)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn fishsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| fishspfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn samenamespfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn fishspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel: &Duel| fishspfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        let code = sc.get_code(duel);
        let filter = aux::necro_valley_filter(|c, duel: &Duel| samenamespfilter(c, duel, e, tp, code));
        let g = duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, 1, None)?;
            if !sg.is_empty() {
                duel.break_effect()?;
                duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.rp == 1 - tp && c.is_previous_controler(duel, tp) && c.is_location(duel, LOCATION_GRAVE)
}
