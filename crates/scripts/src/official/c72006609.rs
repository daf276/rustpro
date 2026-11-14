// Mekk-Knight of the Morning Star
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 72006609;

pub const LISTED_SERIES: &[u16] = &[SET_MEKK_KNIGHT, SET_WORLD_LEGACY];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_MEKK_KNIGHT], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// Link Summoned from Mekk-Knight Avram with another Mekk-Knight (Girsu) to discard and World Legacy cards in the Deck, and monsters
// of the opponent to battle, with Mekk-Knight Spectrum Supreme in the Extra Deck to attack directly.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (38502358, EXTRA),
    (84754430, MZONE),
    (84754430, MZONE),
    (69811710, MZONE),
    (84754430, HAND),
    (69811710, HAND),
    (99674361, DECK),
    (27918365, DECK),
    (93920420, DECK),
    (6103114, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MEKK_KNIGHT]));
    e2.set_value(duel, Value::OptionalCard(tglimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MEKK_KNIGHT]));
    e3.set_value(duel, Value::OptionalCard(tglimit));
    c.register_effect(duel, e3);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_MEKK_KNIGHT]) && c.is_monster(duel)) || c.is_set_card(duel, &[SET_WORLD_LEGACY])) && c.is_discardable(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, costfilter, 1, 1, REASON_DISCARD | REASON_COST)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WORLD_LEGACY]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

// A nil battle target (a direct attack) makes the Lua's IsContains raise an error, which reads as 0.
fn tglimit(duel: &Duel, _: EffectId, c: Option<CardId>) -> i32 {
    let battle_target = c.and_then(|c| c.get_battle_target(duel));
    i32::from(
        c.zip(battle_target)
            .is_some_and(|(c, target)| !c.get_column_group(duel).is_contains(target)),
    )
}
