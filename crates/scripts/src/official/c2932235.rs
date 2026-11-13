// Thousand Umbral Horror
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 2932235;
const CARD_SEVENTH_BARIANS: u32 = 39513225;

pub const LISTED_NAMES: &[u32] = &[CARD_SEVENTH_BARIANS];

// In the hand and on the field, with Seventh Barian's in the hand and Deck to place, in the Graveyard while Number 39: Utopia the
// Envoy of Light (Rank 4) is Xyz Summoned with Mystical Elf and Flying Kamakiri #2.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_SEVENTH_BARIANS, HAND),
    (CARD_SEVENTH_BARIANS, DECK),
    (CARD_SEVENTH_BARIANS, DECK),
    (76504386, EXTRA),
    (76504386, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (15025844, MZONE),
    (3134241, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, aux::faceup_filter(CardId::is_xyz_monster), 1, None))
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CARD_SEVENTH_BARIANS) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None))
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(
        tp,
        |c, duel| plfilter(c, duel, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        duel.place_field_spell(sc, tp)?;
    }
    let c = e.get_handler(duel);
    let reset_turn_count = if duel.is_turn_player(tp) { 2 } else { 1 };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA) && !c.is_xyz_monster(duel));
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, reset_turn_count);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_count(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_XYZ), reset_turn_count);
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, rp: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT > 0 && rp == e.get_owner_player(duel) {
        0
    } else {
        val
    }
}
