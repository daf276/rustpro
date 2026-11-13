// Tindangle Base Gardna
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 94365540;

// On the field and in the hand beside Tindangle Angel, with another in the Deck and the opponent's Gaia Saber (a Link Monster) whose
// linked zones the opponent summons into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (68860936, MZONE),
    (68860936, DECK),
    (68860936, DECK),
    (67598234, MZONE | OPPONENT),
    (67598234, MZONE | OPPONENT),
    (67598234, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SPSUM_PARAM);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_target_range(duel, POS_FACEUP_DEFENSE, 0);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(CardId::is_facedown, tp, LOCATION_MZONE, 0, 1, None)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32) -> bool {
    let seq = c.get_sequence(duel) + if c.is_controler(duel, tp) { 16 } else { 0 };
    zone & (1 << seq) != 0
}

fn lkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_link_monster(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let lg = duel.get_matching_group(lkfilter, tp, 0, LOCATION_MZONE, None);
    let zone = lg.into_iter().fold(0, |zone, tc| zone | tc.get_linked_zone(duel));
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| cfilter(c, duel, tp, zone), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TINDANGLE])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK | POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > -1
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_HAND) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_ATTACK | POS_FACEDOWN_DEFENSE)? != 0
        && tc.is_facedown(duel)
    {
        duel.confirm_cards(1 - tp, tc)?;
    }
    Ok(())
}
