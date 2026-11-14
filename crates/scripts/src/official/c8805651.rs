// Megalith Phuloch
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 8805651;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];
pub const LISTED_NAMES: &[u32] = &[ID];

const MEGALITH_OCH: u32 = 36849933;

struct Params;

impl RitualParams for Params {
    const LOCATION: Option<u32> = Some(LOCATION_GRAVE);
    const FORCEDSELECTION: Option<ForcedSelection> = Some(|duel, e, _, g, _| (g.is_contains(e.get_handler(duel)), false));

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MEGALITH]) && !c.is_code(duel, ID)
    }
}

// On the field, it Ritual Summons Megalith Och from the GY with itself; Och on the field Ritual Summons the copy in the
// hand, which adds Och back from the GY.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (MEGALITH_OCH, GRAVE), (MEGALITH_OCH, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_ritual_summoned(duel));
    e1.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, Cost::hard_once_per_chain::<ID>);
    e2.set_target(duel, Ritual::target::<Params, RITPROC_GREATER>);
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) && c.is_controler(duel, a.tp) {
            Ritual::operation::<Params, RITPROC_GREATER>(duel, a)?;
        }
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEGALITH]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
