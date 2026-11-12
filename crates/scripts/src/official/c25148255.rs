// Junk Anchor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 25148255;

// Copies on the field with Junk Meister (Level 4) in the Graveyard and Drill Warrior in the Extra Deck, and Kojikocy (a
// Level 4 Normal Monster) in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (73218792, GRAVE),
    (73218792, GRAVE),
    (3429238, EXTRA),
    (3429238, EXTRA),
    (1184620, HAND),
    (1184620, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(3429238)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, sccost);
    e1.set_target(duel, sctg);
    e1.set_operation(duel, scop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, 20932152);
    c.register_effect(duel, e2);
}

fn sccost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST)?;
    Ok(false)
}

fn scfilter(c: CardId, duel: &Duel, mg: CardSet, tp: u8) -> bool {
    c.lists_archetype_as_material(duel, &[SET_SYNCHRON])
        && duel.get_location_count_from_ex(tp, tp, Some(mg), c) > 0
        && c.is_synchro_summonable(duel, None, Some(mg))
}

fn mfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    let mg = CardSet::from_iter([c, mc]);
    c.is_set_card(duel, &[SET_JUNK])
        && !c.is_type(duel, TYPE_TUNER)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(|c, duel| scfilter(c, duel, mg, tp), tp, LOCATION_EXTRA, 0, 1, None)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && mfilter(chkc, duel, e, tp, c));
    }
    let filter = |card: CardId, duel: &Duel| mfilter(card, duel, e, tp, c);
    if chk == 0 {
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
    duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mg = CardSet::from_iter([c, tc]);
    let g = duel.get_matching_group(|c, duel| scfilter(c, duel, mg, tp), tp, LOCATION_EXTRA, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.select(duel, tp, 1, 1, None)?;
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
    c.register_effect_full(duel, e1, true);
    let e2 = e1.clone_effect(duel);
    tc.register_effect_full(duel, e2, true);
    if let Some(sc) = sg.get_first() {
        duel.synchro_summon(tp, sc, None, Some(mg))?;
    }
    Ok(())
}
