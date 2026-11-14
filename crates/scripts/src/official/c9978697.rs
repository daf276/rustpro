// X-Saber Pellina
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 9978697;

pub const LISTED_SERIES: &[u16] = &[SET_X_SABER];

struct Destroy;

impl aux::DelayedOperation for Destroy {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.destroy(ag, REASON_EFFECT)?;
        Ok(())
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// X-Saber Anu Piranha in the Deck to summon, and a copy on the field with XX-Saber Emmersblade (Level 3) to Synchro Summon X-Saber Urbellum.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (42737833, MZONE),
    (80108118, EXTRA),
    (80108118, EXTRA),
    (23115241, DECK),
    (23115241, DECK),
    (23115241, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_cost(duel, Cost::self_to_grave);
    e1a.set_target(duel, sptg);
    e1a.set_operation(duel, spop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition_mut(duel, effcon);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_X_SABER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && !c.is_code(duel, ID)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_mzone_count(tp, Some(e.get_handler(duel).into())) >= 2
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 2, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 2, 2, None)?;
    if g.len() == 2 && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 2 {
        aux::delayed_operation_full::<Destroy>(duel, g, PHASE_END, 0, ID, e, tp, aux::stringid(ID, 2), 0);
    }
    Ok(())
}

// The Lua's `not r==REASON_SYNCHRO` is always false, so any material reason passes.
fn effcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(sync) = c.get_reason_card(duel) else { return false };
    if !(c.is_location(duel, LOCATION_GRAVE) && sync.is_set_card(duel, &[SET_X_SABER])) {
        return false;
    }
    a.e.set_label_object_group(duel, Some(sync.into()));
    sync.register_flag_effect(duel, ID, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET), 0, 1);
    true
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(sync) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    if chk == 0 {
        return Ok(sync.has_flag_effect(duel, ID)
            && !(sync.is_has_effect(duel, EFFECT_EXTRA_ATTACK).is_some() && sync.is_has_effect(duel, EFFECT_DIRECT_ATTACK).is_some()));
    }
    sync.create_effect_relation(duel, a.e);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(sync) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if !sync.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let b1 = sync.is_has_effect(duel, EFFECT_EXTRA_ATTACK).is_none();
    let b2 = sync.is_has_effect(duel, EFFECT_DIRECT_ATTACK).is_none();
    if !(b1 || b2) {
        return Ok(());
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    let (desc, code) = if op == Some(1) {
        (3201, EFFECT_EXTRA_ATTACK)
    } else {
        (3205, EFFECT_DIRECT_ATTACK)
    };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, desc);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, code);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sync.register_effect(duel, e1);
    Ok(())
}
