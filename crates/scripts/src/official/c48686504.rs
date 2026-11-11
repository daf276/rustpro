// Lonefire Blossom
use crate::prelude::*;
use crate::testing::{DECK, MZONE};

const ID: u32 = 48686504;

// On the field with Plant monsters (Mushroom Man, Firegrass, Snakeyashi) to Tribute and in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14181608, MZONE),
    (53293545, DECK),
    (53293545, DECK),
    (29802344, DECK),
    (84990171, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, ft: i32, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_PLANT) && (ft > 0 || (c.is_controler(duel, tp) && c.get_sequence(duel) < 5))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |c, duel: &Duel| costfilter(c, duel, ft, tp);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_PLANT) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
