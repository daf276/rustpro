// Artmage Academic Arcane Arts Acropolis
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 74733322;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE];

// s.declared_names[tp], the codes at this key plus tp * 0x100 plus their index and their count at index 0xff.
const DECLARED_NAMES: u64 = (ID as u64) << 16;
const COUNT: u64 = 0xff;

pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE];

// In the hand to activate, with Medius the Pure to Normal Summon in addition, a Spell to discard as cost and Artmage monsters in the
// Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_MEDIUS_THE_PURE, HAND),
    (55144522, HAND),
    (34541940, DECK),
    (60946049, DECK),
    (97434754, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_code(duel, CARD_MEDIUS_THE_PURE));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_cost(duel, |duel, a, chk| {
        Cost::discard_full(duel, a, chk, CardId::is_spell_trap, false, (1, 1))
    });
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| aux::add_values_reset(duel, reset));
}

fn reset(duel: &mut Duel) {
    duel.set_script_state(DECLARED_NAMES | COUNT, 0);
    duel.set_script_state(DECLARED_NAMES | 0x100 | COUNT, 0);
}

fn declared_names(duel: &Duel, tp: u8) -> Vec<u32> {
    let key = DECLARED_NAMES | u64::from(tp) << 8;
    (0..duel.get_script_state(key | COUNT) as u64)
        .map(|index| duel.get_script_state(key | index) as u32)
        .collect()
}

fn declfilter(c: CardId, duel: &Duel, exc1: &[i64], exc2: &[u32]) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE])
        && c.is_monster(duel)
        && c.is_able_to_hand(duel)
        && !exc1.iter().any(|&code| c.is_code(duel, code as u32))
        && !exc2.iter().any(|&code| c.is_code(duel, code))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let faceup_artmage = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ARTMAGE]));
    let fcs = duel
        .get_matching_group(faceup_artmage, tp, LOCATION_MZONE, 0, None)
        .get_class(duel, |c, duel| i64::from(c.get_code(duel)));
    let declared = declared_names(duel, tp);
    let g = duel.get_matching_group(|c, duel| declfilter(c, duel, &fcs, &declared), tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let mut announce_filter = Vec::new();
    for code in g.get_class(duel, |c, duel| i64::from(c.get_code(duel))) {
        announce_filter.extend([code as u64, OPCODE_ISCODE]);
        if announce_filter.len() > 2 {
            announce_filter.push(OPCODE_OR);
        }
    }
    let ac = duel.announce_card(tp, &announce_filter)?;
    let key = DECLARED_NAMES | u64::from(tp) << 8;
    let count = duel.get_script_state(key | COUNT);
    duel.set_script_state(key | count as u64, i64::from(ac));
    duel.set_script_state(key | COUNT, count + 1);
    duel.set_target_param(ac);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_code(duel, code) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let code: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    if !duel.has_flag_effect(tp, ID) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            !(c.is_set_card(duel, &[SET_ARTMAGE]) || c.is_code(duel, CARD_MEDIUS_THE_PURE) || c.is_location(duel, LOCATION_EXTRA))
        });
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}
