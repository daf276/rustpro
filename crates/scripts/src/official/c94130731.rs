// Transcendosaurus Glaciasaurus
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 94130731;

// In the Extra Deck, Synchro Summoned with Dragon Core Hexer (a Level 8 Tuner) and Kojikocy (Level 4), with Normal Monsters in the Graveyard
// and the opponent's Dark Hole in hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (38955728, MZONE),
    (38955728, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_race(duel, RACE_DINOSAUR) && c.is_summon_location(duel, LOCATION_GRAVE)
    });
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, tdtg);
    e4.set_operation(duel, tdop);
    c.register_effect(duel, e4);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.hint_selection_full(g, true);
    let c = e.get_handler(duel);
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && c.is_relate_to_effect(duel, e)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && aux::nvfilter(c, duel)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
