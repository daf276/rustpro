// Virakam, the Divine Spirit of Artifice
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 10673071;

pub const LISTED_NAMES: &[u32] = &[CARD_INVOCATION];

// In hand with Aleister the Invoker on the field, Invocation in the Deck to Set, and on the field with Invoked Raidjin to
// activate an effect that the opponent's Elfnote Lucina responds to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (86120751, MZONE),
    (CARD_INVOCATION, DECK),
    (49513164, MZONE),
    (13597785, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_REMOVE + CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, discon);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ALEISTER]) && c.is_monster(duel) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_INVOCATION) || (c.is_spell(duel) && c.lists_code(duel, CARD_INVOCATION))) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SET, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let chainlink = i32::from(duel.get_current_chain_full(true)) - 1;
    if !(chainlink > 0 && duel.is_chain_disablable(a.ev as u8) && a.ep == 1 - a.tp) {
        return false;
    }
    let chainlink = chainlink as u8;
    let trig_player: u8 = duel.get_chain_info(chainlink, CHAININFO_TRIGGERING_PLAYER);
    let trig_type: u32 = duel.get_chain_info(chainlink, CHAININFO_TRIGGERING_TYPE);
    trig_player == a.tp && trig_type & TYPE_FUSION > 0
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel)
            && (rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT)
                || (!relation && duel.is_player_can_remove(tp, None))));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    if relation {
        rc.create_effect_relation(duel, e);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(CardSet::from_iter([c, rc])), 1, tp, 0);
    } else {
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, tp, 0);
        let (controler, location) = (rc.get_previous_controler(duel), rc.get_previous_location(duel));
        duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, controler, location as i32);
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, ev) = (a.e, a.ev as u8);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    if c.is_relate_to_effect(duel, e)
        && duel.remove(c, 0, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_REMOVED)
        && duel.negate_effect(ev)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && let Some(eg) = a.eg
    {
        duel.remove(eg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
