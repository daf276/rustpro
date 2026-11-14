// Rikka Konkon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 76869711;

pub const LISTED_SERIES: &[u16] = &[SET_RIKKA];
// In hand and the Field Zone with Rikka monsters (Mudan) on the field, Rikka Spells/Traps (Glamour, Sheet) in the Deck
// to Set, and Mudan and Glamour in hand to Tribute the opponent's monsters (Elfnote Lucina).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, 0),
    (ID, 0),
    (71002019, MZONE),
    (71002019, MZONE),
    (71002019, HAND),
    (71002019, HAND),
    (69164989, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (69164989, DECK),
    (68941332, DECK),
    (68941332, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_RIKKA])),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, CARD_RIKKA_KONKON);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    c.register_effect(duel, e3);
    let e3a = e3.clone_effect(duel);
    e3a.set_code(duel, EFFECT_EXTRA_RELEASE_NONSUM);
    e3a.set_count_limit(duel, 1, 0);
    e3a.set_value(duel, Value::Reason(relval));
    c.register_effect(duel, e3a);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RIKKA]) && c.is_spell_trap(duel) && !c.is_type(duel, TYPE_FIELD) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let sg = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !sg.is_empty() {
        duel.sset(tp, sg)?;
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_PLANT));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn relval(duel: &Duel, _: EffectId, re: Option<EffectId>, r: u32, _: u8) -> bool {
    re.is_some_and(|re| re.is_activated(duel) && r & REASON_COST != 0 && re.get_handler(duel).is_set_card(duel, &[SET_RIKKA]))
}
