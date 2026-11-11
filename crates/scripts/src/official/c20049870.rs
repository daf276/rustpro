// GMX Researcher Selande
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 20049870;

// A second copy in the hand and GMX Partner Selandea to Special Summon it by a monster effect, GMX Chairman Kimridge and
// Dinosaur Normal Monsters (Mammoth Graveyard, Trakodon) in the Deck, and GMX Applied Experiment #55 to search. No Dinosaur
// Fusion Monster is ported, so the third effect is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (2618725, HAND),
    (56034579, DECK),
    (40374923, DECK),
    (42348802, DECK),
    (18795635, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.re.is_some_and(|re| re.is_monster_effect(duel)));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, discon);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    ((c.is_race_except(duel, RACE_WARRIOR) && c.is_set_card(duel, &[SET_GMX]))
        || (c.is_level_below(duel, 3) && c.is_race(duel, RACE_DINOSAUR)))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GMX]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn disconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DINOSAUR) && c.is_fusion_monster(duel) && c.is_faceup(duel)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(disconfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel)
            && duel.is_existing_matching_card(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
        let g = duel.select_matching_card(tp, CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.hint_selection(sc);
            sc.negate_effects(duel, c, RESET_PHASE | PHASE_END, true);
        }
    }
    Ok(())
}
