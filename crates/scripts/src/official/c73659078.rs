// Snowdust Giant
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 73659078;

pub const COUNTER_LIST: &[u16] = &[COUNTER_ICE];
const COUNTER_ICE: u16 = 0x1015;

// Twice in the Extra Deck, Xyz Summoned with Level 4 WATER Normal Monsters, with more in the hand to reveal.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, MZONE),
    (2118022, HAND),
    (2483611, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, countertg);
    e1.set_operation(duel, counterop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| c.is_attribute_except(duel, ATTRIBUTE_WATER));
    e2.set_value(duel, Value::Card(|duel, _, _| duel.get_counter(0, 1, 1, COUNTER_ICE) as i32 * -200));
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && !c.is_public(duel)
}

fn countertg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    Ok(false)
}

fn counterop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(|c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let hg = duel.get_matching_group(cfilter, tp, LOCATION_HAND, 0, None);
    if hg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let rg = hg.select(duel, tp, 1, hg.len() as u8, None)?;
    duel.confirm_cards(1 - tp, rg)?;
    duel.shuffle_hand(tp)?;
    for _ in 0..rg.len() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_COUNTER);
        if let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            tc.add_counter(duel, COUNTER_ICE, 1);
        }
    }
    Ok(())
}
