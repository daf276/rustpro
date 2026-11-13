// Dark Creator
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 52768390;
const TOKEN_DARK_CREATOR: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[TOKEN_DARK_CREATOR];

// Copies in the hand and on the field, with Fiends (Kozaky, White Duston) on the field to Tribute and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (99171160, MZONE),
    (99171160, MZONE),
    (3557275, MZONE),
    (99171160, MZONE | OPPONENT),
    (99171160, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::SpecialSummonedCode(TOKEN_DARK_CREATOR),
];

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 3000, 1000, 10, RACE_FIEND, ATTRIBUTE_DARK)
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, tokensptg);
    e3.set_operation(duel, tokenspop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_BATTLE_DESTROYED);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, stock::damage);
    c.register_effect(duel, e4);
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, |rc, duel| rc.is_race(duel, RACE_FIEND | RACE_FAIRY), 2, release_options(c, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(
        tp,
        |rc, duel| rc.is_race(duel, RACE_FIEND | RACE_FAIRY),
        (2, 2),
        true,
        release_options(c, tp),
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST)?;
    }
    Ok(())
}

fn tokensptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > 0 && duel.is_player_can_special_summon_monster(tp, TOKEN_DARK_CREATOR, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, ft as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, ft as u8, tp, 0);
    Ok(false)
}

fn tokenspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft > 0 && duel.is_player_can_special_summon_monster(tp, TOKEN_DARK_CREATOR, token_stats()) {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        let mut sg = CardSet::default();
        for _ in 0..ft {
            let Some(token) = duel.create_token(tp, TOKEN_DARK_CREATOR) else {
                continue;
            };
            duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
            sg.insert(token);
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3207);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            token.register_effect_full(duel, e1, true);
            let e2 = e1.clone_effect(duel);
            e2.set_description(duel, 3000);
            e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
            e2.set_value(duel, 1);
            token.register_effect_full(duel, e2, true);
        }
        if c.is_relate_to_effect(duel, e) && !sg.is_empty() {
            let e3a = duel.create_effect(c);
            e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
            e3a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e3a.set_code(duel, EVENT_LEAVE_FIELD_P);
            e3a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_DESTROY));
            e3a.set_operation(duel, |duel, a| {
                let (c, tp) = (a.e.get_handler(duel), a.e.get_label(duel));
                let e3b = duel.create_effect(c);
                e3b.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
                e3b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e3b.set_code(duel, EVENT_DESTROYED);
                e3b.set_operation(duel, |duel, a| {
                    let e = a.e;
                    let dg = e
                        .get_label_object_group(duel)
                        .unwrap_or_default()
                        .filter(duel, CardId::is_on_field, None);
                    if !dg.is_empty() {
                        duel.hint(HINT_CARD, 1 - e.get_label(duel) as u8, u64::from(ID));
                        duel.destroy(dg, REASON_EFFECT)?;
                    }
                    e.reset(duel);
                    Ok(())
                });
                e3b.set_label(duel, tp);
                e3b.set_label_object_group(duel, a.e.get_label_object_group(duel));
                c.register_effect(duel, e3b);
                Ok(())
            });
            e3a.set_label(duel, i64::from(tp));
            e3a.set_label_object_group(duel, Some(sg));
            e3a.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect(duel, e3a);
        }
        duel.special_summon_complete()?;
    }
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e4.set_target_range(duel, 1, 0);
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e4, tp);
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(700);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 700);
    Ok(false)
}
