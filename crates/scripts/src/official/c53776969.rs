// Scareclaw Light-Heart
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 53776969;

pub const LISTED_SERIES: &[u16] = &[SET_SCARECLAW];

const REICHPHOBIA: u32 = 56063182;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, st: u64, tp: u8) -> bool {
        c.is_in_main_mzone(duel, Some(tp))
            && (c.is_set_card_full(duel, &[SET_SCARECLAW], Some(lc), st, tp)
                || c.is_summon_code(duel, Some(lc), st, tp, &[CARD_VISAS_STARFROST]))
    }
}

pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST, REICHPHOBIA];
// In the Extra Deck and Graveyard, Link Summoned with Visas Starfrost, with Primitive Planet Reichphobia in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (CARD_VISAS_STARFROST, MZONE),
    (CARD_VISAS_STARFROST, MZONE),
    (REICHPHOBIA, DECK),
    (REICHPHOBIA, DECK),
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
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_link_summoned(duel) && c.is_in_extra_mzone(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, REICHPHOBIA) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_VISAS_STARFROST));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, 0, 1, None)
}
