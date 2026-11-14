// Mannadium Reframing
use crate::prelude::*;
use crate::testing::{Check, MZONE, SZONE};

const ID: u32 = 18158393;

pub const LISTED_SERIES: &[u16] = &[SET_MANNADIUM];
pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST];
// Set with a Synchro Monster, Black Rose Dragon, on the field. Its destruction needs Visas Starfrost or a monster with 1500
// ATK and 2100 DEF, and its second effect a "Mannadium" monster in the Graveyard; none of them is ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (73580471, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) || re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_SYNCHRO)),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_possible_operation_info(0, CATEGORY_DESTROY, a.eg, 1, a.tp, 0);
        }
    }
    Ok(false)
}

fn vsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && (c.is_code(duel, CARD_VISAS_STARFROST) || (c.is_monster(duel) && c.is_attack(duel, &[1500]) && c.is_defense(duel, &[2100])))
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(()) };
    let rc = re.get_handler(duel);
    if duel.negate_activation(a.ev as u8)
        && rc.is_relate_to_effect(duel, re)
        && duel.is_existing_matching_card(vsfilter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
        && rc.is_destructable(duel)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_MANNADIUM]) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 3, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
