// Tribe-Infecting Virus
use crate::prelude::*;
use crate::testing::MZONE;

const ID: u32 = 33184167;

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let race = g.into_iter().fold(0, |race, tc| race | tc.get_race(duel));
    duel.hint(HINT_SELECTMSG, tp, 0);
    let arc = duel.announce_race(tp, 1, race)?;
    a.e.set_label(duel, arc as i64);
    let dg = g.filter(duel, |c, duel| c.is_race(duel, arc), None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let arc = a.e.get_label(duel) as u64;
    let g = duel.get_matching_group(
        |c, duel| c.is_faceup(duel) && c.is_race(duel, arc),
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
