// D/D Savant Copernicus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT, PZONE};

const ID: u32 = 46796664;

pub const LISTED_SERIES: &[u16] = &[SET_DD, SET_DARK_CONTRACT];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand to Normal Summon, with D/D cards and Dark Contract with the Witch in the Deck to send, and one in a
// Pendulum Zone against the opponent's Hinotama.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, PZONE),
    (46130346, HAND | OPPONENT),
    (46130346, HAND | OPPONENT),
    (72181263, DECK),
    (9765723, DECK),
    (33334269, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        !c.is_set_card(duel, &[SET_DD]) && sumtype & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_condition(duel, discon);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    duel.is_chain_negatable(a.ev as u8)
        && a.re.is_some_and(|re| re.is_spell_effect(duel))
        && aux::damcon1(duel, a)
        && a.e.get_handler(duel).get_flag_effect(duel, ID) == 0
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !duel.select_effect_yes_no(a.tp, c)? {
        return Ok(());
    }
    c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    if !duel.negate_effect(a.ev as u8) {
        return Ok(());
    }
    duel.break_effect()?;
    duel.destroy(c, REASON_EFFECT)?;
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_DD]) || c.is_set_card(duel, &[SET_DARK_CONTRACT])) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
