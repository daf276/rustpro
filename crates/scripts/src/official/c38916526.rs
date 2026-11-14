// Rafale, Champion Fur Hire
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 38916526;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
pub const LISTED_NAMES: &[u32] = &[ID];
// Revived by Mayhem Fur Hire beside other Fur Hire monsters, with Fur Hire monsters in the hand to discard against the
// opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (91405870, HAND),
    (93850652, MZONE),
    (47647354, MZONE),
    (93850652, HAND),
    (28004531, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, negcost));
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_code(duel, ID) && c.is_faceup(duel) && c.is_set_card(duel, &[SET_FUR_HIRE])
}

fn excavated_count(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group(ctfilter, tp, LOCATION_MZONE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let ct = excavated_count(duel, tp);
        if duel.get_field_group_count(tp, LOCATION_DECK, 0) < ct {
            return Ok(false);
        }
        let g = duel.get_decktop_group(tp, ct);
        return Ok(g.filter_count(duel, CardId::is_able_to_hand, None) > 0);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = excavated_count(duel, tp);
    duel.confirm_decktop(tp, ct)?;
    let g = duel.get_decktop_group(tp, ct);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.disable_shuffle_check(true);
        if sg.get_first().is_some_and(|sc| sc.is_able_to_hand(duel)) {
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
            duel.shuffle_hand(tp)?;
        } else {
            duel.send_to_grave(sg, REASON_RULE)?;
        }
        if ct > 1 {
            duel.shuffle_deck(tp)?;
        }
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.ep != a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_discardable(duel)
}

pub fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

pub fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8)
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
