// Dogmatika Alba Zoa
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 51522296;

const EXTRA_TYPES: u32 = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;

pub const LISTED_NAMES: &[u32] = &[31002402];
pub const LISTED_SERIES: &[u16] = &[SET_DOGMATIKA];
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(ritlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_DOGMATIKA]));
    e2.set_value(duel, Value::Effect(unaval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOEXTRA);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, applytg);
    e3.set_operation(duel, applyop);
    c.register_effect(duel, e3);
}

fn ritlimit(duel: &Duel, _: EffectId, se: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    if st & SUMMON_TYPE_RITUAL == SUMMON_TYPE_RITUAL {
        return se.is_some_and(|se| se.get_handler(duel).is_set_card(duel, &[SET_DOGMATIKA]));
    }
    true
}

fn unaval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_monster_effect(duel)
        && te.is_activated(duel)
        && te.get_owner_player(duel) == 1 - e.get_handler_player(duel)
        && te.is_active_type(duel, EXTRA_TYPES)
}

// b1, b2 and the opponent's face-up Extra Deck monsters, as both the target and the operation compute them.
fn options(duel: &Duel, tp: u8) -> (usize, bool, bool, CardSet) {
    let ex_ct = duel.get_field_group_count(tp, 0, LOCATION_EXTRA);
    let b1 = ex_ct >= 2 && duel.is_existing_matching_card(CardId::is_able_to_grave, tp, 0, LOCATION_HAND | LOCATION_EXTRA, ex_ct / 2, None);
    let texg = duel.get_matching_group(
        |c, duel| c.is_faceup(duel) && c.is_type(duel, EXTRA_TYPES),
        tp,
        0,
        LOCATION_MZONE,
        None,
    );
    let b2 = !texg.is_empty() && texg.len() == texg.filter_count(duel, CardId::is_able_to_extra, None);
    (ex_ct, b1, b2, texg)
}

fn applytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let (_, b1, b2, _) = options(duel, tp);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, (LOCATION_HAND | LOCATION_EXTRA) as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn applyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let (ex_ct, b1, b2, mut texg) = options(duel, tp);
    if !(b1 || b2) {
        return Ok(());
    }
    let op = duel.select_effect(1 - tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    if op == Some(1) {
        let send_ct = (ex_ct / 2) as u8;
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(
            1 - tp,
            CardId::is_able_to_grave,
            tp,
            0,
            LOCATION_HAND | LOCATION_EXTRA,
            send_ct,
            send_ct,
            None,
        )?;
        if !g.is_empty() {
            duel.send_to_grave_full(g, REASON_EFFECT, PLAYER_NONE, 1 - tp)?;
        }
    } else if op == Some(2) {
        texg.keep_matching(duel, CardId::is_able_to_extra, None);
        duel.send_to_deck_full(texg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT, 1 - tp)?;
    }
    Ok(())
}
