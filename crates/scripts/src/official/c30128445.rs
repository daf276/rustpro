// Transcendosaurus Drillygnathus
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE, OPPONENT, REMOVED};

const ID: u32 = 30128445;

// In the Extra Deck, Xyz Summoned with Level 6 Normal Monsters (Frostosaurus, Sword Arm of Dragon), with banished Dinosaurs
// (Uraby) to Special Summon, Normal Monsters in the Graveyard to shuffle and Raigeki and Dark Hole on the opponent's side to
// destroy it. The doubled battle damage needs a battle with no materials left, which random play rarely reaches.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (6631034, MZONE),
    (13069066, MZONE),
    (13069066, MZONE),
    (6631034, MZONE),
    (1784619, REMOVED),
    (1784619, REMOVED),
    (549481, GRAVE),
    (1184620, GRAVE),
    (12580477, DECK | OPPONENT),
    (53129443, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<6, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_condition(duel, damcon);
    e1.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_overlay_count(duel) == 0 && c.get_battle_target(duel).is_some()
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    let c = e.get_handler(duel);
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_DECK))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && c.is_relate_to_effect(duel, e)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && aux::nvfilter(c, duel)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
