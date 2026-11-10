// Set Rotation
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 73468603;

// In the hand with two different Field Spells (Umi, Necrovalley) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (22702055, DECK),
    (22702055, DECK),
    (47355498, DECK),
    (47355498, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FIELD) && c.is_ssetable(duel)
}

fn class_count(duel: &Duel, g: CardSet) -> usize {
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(filter, a.tp, LOCATION_DECK, 0, None);
    Ok(chk == 0 && class_count(duel, g) > 1)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let mut g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if class_count(duel, g) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    let tg1 = g.select(duel, tp, 1, 1, None)?;
    let tc1 = tg1.get_first();
    let code = tc1.map_or(0, |tc| tc.get_code(duel));
    g.remove_matching(duel, |c, duel| c.is_code(duel, code), None);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let tg2 = g.select(duel, tp, 1, 1, None)?;
    duel.sset(tp, tg1)?;
    duel.sset_full(tp, tg2, 1 - tp, true)?;
    for tc in [tg1.get_first(), tg2.get_first()].into_iter().flatten() {
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 1);
    e1.set_condition(duel, con);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| {
            re.is_active_type(duel, TYPE_FIELD)
                && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
                && re.get_handler(duel).get_flag_effect(duel, ID) == 0
        }),
    );
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SSET);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_condition(duel, con);
    e2.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_FIELD) && c.get_flag_effect(duel, ID) == 0);
    duel.register_effect(e2, tp);
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.get_flag_effect(duel, ID) != 0
}

fn con(duel: &Duel, _: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, 0, LOCATION_FZONE, LOCATION_FZONE, 1, None)
}
