// Protector Whelp of the Destruction Swordsman
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 47158777;

pub const LISTED_SERIES: &[u16] = &[SET_DESTRUCTION_SWORD, SET_BUSTER_BLADER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// Link Summoned from Buster Whelp and Dragon Buster Destruction Sword (Destruction Sword) with more in the Deck, Buster Blader in
// the hand to Special Summon, and Raigeki to empty the opponent's field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (49823708, MZONE),
    (76218313, MZONE),
    (49823708, MZONE),
    (76218313, DECK),
    (49823708, DECK),
    (78193831, HAND),
    (78193831, MZONE),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, dmgcon);
    e2.set_target(duel, dmgtg);
    e2.set_operation(duel, dmgop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DESTRUCTION_SWORD]) && c.is_able_to_grave(duel)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BUSTER_BLADER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g1 = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let g2 = duel.get_matching_group(|c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
    if !g1.is_empty()
        && duel.send_to_grave(g1, REASON_EFFECT)? > 0
        && g1.get_first().is_some_and(|c| c.is_location(duel, LOCATION_GRAVE))
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && !g2.is_empty()
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g3 = g2.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.special_summon(g3, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn dmgcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_turn_player(tp) && duel.get_field_group_count(tp, 0, LOCATION_MZONE) == 0
}

fn dmgfilter(c: CardId, duel: &Duel) -> bool {
    c.get_attack_announced_count(duel) == 0 && c.is_faceup(duel) && c.is_set_card(duel, &[SET_BUSTER_BLADER])
}

fn dmgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_target(dmgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, dmgfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let attack = g.and_then(|g| g.get_first()).map_or(0, |c| c.get_attack(duel));
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, attack);
    Ok(false)
}

fn dmgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.damage(1 - tp, tc.get_attack(duel).max(0) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
