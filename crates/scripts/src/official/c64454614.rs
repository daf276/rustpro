// Arcana Extra Joker
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 64454614;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_WARRIOR, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        lcheck(duel, sg, lc, sumtype, tp)
    }
}

// In the Extra Deck and on the field, Link Summoned from three different Warriors, with Level 4 Warriors in the Deck to Special Summon and
// add, the opponent's Mind Control to target it against Spells in hand to discard, and Blue-Eyes White Dragons to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (91152256, MZONE),
    (48305365, MZONE),
    (11321183, MZONE),
    (91152256, DECK),
    (48305365, DECK),
    (11321183, DECK),
    (91152257, DECK),
    (91152258, DECK),
    (98234196, DECK),
    (5318639, HAND),
    (5318639, HAND),
    (37520316, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, negcost);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn lcheck(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
    aux::check_different_property(duel, g, &|c, duel| c.get_codes_full(duel, Some(lc), sumtype, tp))
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let mut lg = c.get_linked_group(duel);
    lg.insert(c);
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| lg.is_exists(duel, |c, _| tg.is_contains(c), 1, None)) && duel.is_chain_negatable(a.ev as u8)
}

fn costfilter(c: CardId, duel: &Duel, tpe: u32) -> bool {
    c.is_type(duel, tpe) && c.is_discardable(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(false) };
    let rtype = re.get_active_type(duel) & (TYPE_MONSTER | TYPE_SPELL | TYPE_TRAP);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| costfilter(c, duel, rtype), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, |c, duel| costfilter(c, duel, rtype), 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE) && c.is_link_summoned(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL)
        && c.is_level(duel, &[4])
        && c.is_race(duel, RACE_WARRIOR)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[4]) && c.is_race(duel, RACE_WARRIOR) && c.is_able_to_hand(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g1 = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g1.is_empty() && duel.special_summon(g1, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        duel.confirm_cards(1 - tp, g1)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g2 = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g2.is_empty() {
            duel.send_to_hand(g2, tp, REASON_EFFECT)?;
        }
    }
    Ok(())
}
