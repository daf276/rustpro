// Fleeting Phantom Mask Master
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 44694191;

// In the Graveyard and the hand, with Dark Hole in the Graveyard to banish and Ryu-Kishin, a Fiend Normal Monster, in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, HAND),
    (53129443, GRAVE),
    (53129443, GRAVE),
    (15303296, DECK),
    (15303296, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOGRAVE);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_target(duel, tgtg);
    e1a.set_operation(duel, tgop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, e, c| {
        let handler = e.get_handler(duel);
        c == handler || Some(c) == handler.get_battle_target(duel)
    });
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit_full(duel, 1, (ID, 1), EFFECT_COUNT_CODE_DUEL);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, stock::special_summon_self_target);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND | RACE_ILLUSION | RACE_SPELLCASTER) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(sc) = sc
        && duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
    {
        let original_type = sc.get_original_race(duel);
        if c.is_relate_to_effect(duel, e)
            && c.is_faceup(duel)
            && c.is_race_except(duel, original_type)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.break_effect()?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_RACE);
            e1.set_value(duel, original_type as i32);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_remove_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, spcostfilter, LOCATION_GRAVE)
}
