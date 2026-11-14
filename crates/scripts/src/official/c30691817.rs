// Marincess Sea Angel
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 30691817;

pub const LISTED_SERIES: &[u16] = &[SET_MARINCESS];
pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        mfilter(c, duel, lc, sumtype, tp)
    }
}

// Link Summoned from Marincess Sea Star, with Marincess Dive in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62886670, MZONE),
    (62886670, MZONE),
    (57329501, DECK),
    (57329501, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_target(duel, tg);
    e2.set_operation(duel, op);
    c.register_effect(duel, e2);
}

fn mfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_MARINCESS], Some(lc), sumtype, tp) && c.is_level_below(duel, 4)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        c.is_code(duel, ID) && sumtype & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK
    });
    duel.register_effect(e1, tp);
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MARINCESS]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
