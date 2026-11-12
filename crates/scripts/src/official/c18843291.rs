// Minerva, the Athenian Lightsworn
use super::c90587641::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 18843291;

// In the Extra Deck, Synchro Summoned from Felis (a Level 4 Tuner) and Jain (Level 4), with Lightsworn monsters of different
// Types in the Deck to send and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (73176465, MZONE),
    (96235275, MZONE),
    (22624373, DECK),
    (22624373, DECK),
    (58996430, DECK),
    (21502796, DECK),
    (96576187, GRAVE),
    (96576187, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_REMOVE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 1);
    e1.set_player_card_target(duel, rmlimit);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE);
    e2a.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2a.set_value(duel, Value::MaterialCheck(valcheck));
    e2a.set_label_object(duel, Some(e2));
    c.register_effect(duel, e2a);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DECKDES);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, toptgcost);
    e3.set_target(duel, toptgtg);
    e3.set_operation(duel, |duel, a| {
        let ct = a.e.get_label(duel) as u16;
        duel.discard_deck(a.tp, ct, REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e3);
}

fn rmlimit(duel: &Duel, e: EffectId, c: CardId, _: u8, r: u32) -> bool {
    c.is_set_card(duel, &[SET_LIGHTSWORN])
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_faceup(duel)
        && c.is_controler(duel, e.get_handler_player(duel))
        && !c.is_immune_to_effect(duel, e)
        && r & REASON_EFFECT > 0
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIGHTSWORN]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(e.get_label(duel) > 0 && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(tgfilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let ct = (e.get_label(duel) as usize).min(g.len());
    let tg = aux::select_unselect_group(
        duel,
        g,
        1,
        ct,
        &aux::dpcheck(|c, duel| c.get_race(duel) as i64),
        tp,
        HINTMSG_TOGRAVE,
    )?;
    if !tg.is_empty() {
        duel.send_to_grave(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let player = e.get_handler_player(duel);
    let ct = c.get_material(duel).filter_count(
        duel,
        |mc, duel| mc.is_set_card_full(duel, &[SET_LIGHTSWORN], Some(c), SUMMON_TYPE_SYNCHRO as u64, player),
        None,
    );
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, ct as i64);
    }
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIGHTSWORN]) && c.is_monster(duel) && c.is_able_to_remove_as_cost(duel)
}

fn toptgcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_player_can_discard_deck_as_cost(tp, 1) && duel.is_existing_matching_card(costfilter, tp, LOCATION_GRAVE, 0, 1, None)
        );
    }
    let ct = 1 + (2..=4).filter(|&i| duel.is_player_can_discard_deck_as_cost(tp, i)).count();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_GRAVE, 0, 1, ct as u8, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    e.set_label(duel, g.len() as i64);
    Ok(false)
}

fn toptgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let ct = a.e.get_label(duel) as i32;
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, ct);
    Ok(false)
}
