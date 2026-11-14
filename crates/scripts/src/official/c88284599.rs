// Sauravis, Dragon Sage of the Voiceless Voice
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 88284599;

struct SendToDeck;

impl aux::DelayedOperation for SendToDeck {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_deck(ag, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        Ok(())
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        i64::from(duel.get_turn_count()) == duel.get_script_state(crate::keys::delayed_turn(ID, a.e.get_label(duel))) + 1
    }
}

// In the hand with Prayers of the Voiceless Voice and Spells to shuffle, and on the field with LIGHT Ritual Monsters in the
// Deck to summon when the opponent activates Dark Hole or Mage Power.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (52472775, HAND),
    (52472775, GRAVE),
    (53129443, GRAVE),
    (10774240, DECK),
    (56350972, DECK),
    (53129443, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e2.set_cost(duel, Cost::self_to_hand);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_deck_as_cost(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.is_exists(duel, CardId::is_ritual_spell, 1, None), false)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let rg = duel.get_matching_group(cfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, Some(a.e.get_handler(duel).into()));
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && aux::select_unselect_group_check(duel, rg, 2, 2, &rescon)
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(cfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, Some(e.get_handler(duel).into()));
    let g = aux::select_unselect_group_full(duel, rg, 2, 2, &rescon, tp, HINTMSG_TODECK, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.hint_selection_full(g, true);
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_ritual_monster(duel)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_WARRIOR | RACE_DRAGON)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, true)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_DECK;
    if chk == 0 {
        return Ok(duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    let sc = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon(sc, 0, tp, tp, false, true, POS_FACEUP)? > 0
    {
        let turn_count = i64::from(duel.get_turn_count());
        let reset = RESET_PHASE | PHASE_END;
        let hint = aux::stringid(ID, 2);
        if let Some(delayed) = aux::delayed_operation_full::<SendToDeck>(duel, sc.into(), PHASE_END, reset, ID, e, tp, hint, 2) {
            let key = crate::keys::delayed_turn(ID, delayed.get_label(duel));
            duel.set_script_state(key, turn_count);
        }
    }
    Ok(())
}
