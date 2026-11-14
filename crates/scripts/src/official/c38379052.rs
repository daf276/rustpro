// Argostars - Slayer Eteo
use super::c65889305::{trap_sptg, trap_summon};
use super::c91284003::argosfilter;
use crate::prelude::*;
use crate::testing::{Check, REMOVED, SZONE};

const ID: u32 = 38379052;

pub const LISTED_SERIES: &[u16] = &[SET_ARGOSTARS];
// Set copies to answer the opponent's activations, and a banished Argostars monster.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (ID, SZONE), (91438674, REMOVED)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_WARRIOR));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, |_, a, _| Ok(a.ep == 1 - a.tp));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    trap_sptg::<ID, 800>(duel, a, chk, (CATEGORY_TOHAND, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if trap_summon::<ID, 800>(duel, a)?.is_some_and(|count| count > 0)
        && duel.is_existing_matching_card(argosfilter, tp, LOCATION_REMOVED, 0, 1, None)
    {
        let g = duel.get_matching_group(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if !sg.is_empty() {
            duel.hint_selection(sg);
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
