// Long-Tailed Black Horse
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 92447211;

// Copies and Zombies (Skull Servant) in the hand, with EARTH Zombies (Zombino) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (32274490, HAND),
    (32274490, HAND),
    (14575467, DECK),
    (14575467, DECK),
    (14575467, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_ATKCHANGE);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[revealcost, tgcost]));
    e1a.set_target(duel, tgtg);
    e1a.set_operation(duel, tgop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
}

fn revealfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
    c.is_race(duel, RACE_ZOMBIE) && c.is_able_to_grave(duel)
}

fn revealcostop(duel: &mut Duel, a: &Args, g: CardSet) {
    if let Some(rc) = g.first() {
        a.e.set_label_object_group(duel, Some(rc.into()));
        rc.create_effect_relation(duel, a.e);
    }
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Cost::reveal(duel, a, chk, revealfilter, false, (1, 1), Some(revealcostop), LOCATION_HAND)
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH) && c.is_race(duel, RACE_ZOMBIE) && c.is_able_to_grave_as_cost(duel)
}

fn tgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_DECK)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, a.e.get_label_object_group(duel), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(a.e.get_handler(duel).into()), 1, tp, 500);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(rc) = e.get_label_object_group(duel).and_then(|g| g.first()) else {
        return Ok(());
    };
    if rc.is_relate_to_effect(duel, e) {
        duel.send_to_grave(rc, REASON_EFFECT)?;
        if rc.is_location(duel, LOCATION_GRAVE) && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            c.update_attack(duel, 500, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None);
        }
    }
    Ok(())
}
