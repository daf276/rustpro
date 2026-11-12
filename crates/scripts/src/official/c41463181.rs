// Salamangreat Heatleo
use super::c31313405::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 41463181;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Link Summoned from FIRE Effect Monsters, with the opponent's Set Spell to shuffle and a Normal Monster in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (7925734, MZONE),
    (9742784, MZONE),
    (12580477, SZONE | OPPONENT),
    (13179332, GRAVE),
];
// The second effect needs a Reincarnation Link Summon, out of reach without a second copy used as material.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    aux::enable_check_reincarnation(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, atkcon);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_STZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_deck(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_deck, tp, 0, LOCATION_STZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, CardId::is_able_to_deck, tp, 0, LOCATION_STZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reincarnation_summoned(duel) && c.is_link_summoned(duel)
}

fn filter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_attack(duel);
    c.is_monster(duel) && duel.is_existing_target(|c, duel| filter2(c, duel, atk), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

fn filter2(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) != atk
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter1(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELF);
    let g1 = duel.select_target(tp, |c, duel| filter1(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    a.e.set_label_object_group(duel, g1);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let atk = g1.and_then(|g| g.get_first()).map_or(0, |gc| gc.get_attack(duel));
    duel.select_target(tp, |c, duel| filter2(c, duel, atk), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let gc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let mut targets = g.iter();
    let mut tc = targets.next();
    if tc == gc {
        tc = targets.next();
    }
    unported!(duel, gc.is_none() || tc.is_none());
    let (Some(gc), Some(tc)) = (gc, tc) else { return Ok(()) };
    if gc.is_faceup(duel) && gc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, gc.get_attack(duel));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
