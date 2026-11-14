// Drytron Alpha Thuban
use engine::{OperationFn, TargetFn};

use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 97148796;

pub const LISTED_SERIES: &[u16] = &[SET_DRYTRON];
// Drytron monsters to Tribute, and Drytron Meteonis Draconids, a Ritual Monster, to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (96026108, MZONE),
    (60037599, HAND),
    (69815951, DECK),
    (69815951, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let category = CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH;
    register::<ID>(duel, c, category, sptg::<CATEGORY_TOHAND, { LOCATION_DECK as i32 }>, spop);
}

// The effects Alpha Thuban, Zeta Aldhibah, Gamma Eltanin and Delta Altais share.
pub fn register<const ID: u32>(duel: &mut Duel, c: CardId, category: u64, sptg: TargetFn, spop: OperationFn) {
    c.enable_unsummonable(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(spconlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, category);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Drytron::tribute_cost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(|duel, c| !c.is_summonable_card(duel)));
}

fn spconlimit(duel: &Duel, _: EffectId, se: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    se.is_some_and(|se| se.is_has_type(duel, EFFECT_TYPE_ACTIONS) && se.get_handler(duel).is_set_card(duel, &[SET_DRYTRON]))
}

// sptg, with the category and parameter of its possible operation.
pub fn sptg<const CATEGORY: u64, const PARAM: i32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned_full(duel, a.e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    let location = c.get_location(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, location as i32);
    duel.set_possible_operation_info(0, CATEGORY, None, 1, tp, PARAM);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_monster(duel) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    spop_search::<ID>(duel, a, thfilter)
}

// spop of Alpha Thuban and Zeta Aldhibah, which add a card matching thfilter.
pub fn spop_search<const ID: u32>(duel: &mut Duel, a: &Args, thfilter: fn(CardId, &Duel) -> bool) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}
