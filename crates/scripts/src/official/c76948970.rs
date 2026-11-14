// Mitsurugi no Miko, Wousu
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 76948970;

// Copies and Mitsurugi no Mikoto, Saji (a Reptile) in the hand to release with an opponent's monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (18176525, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_HANDES + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_race(duel, RACE_REPTILE) || c.is_controler(duel, 1 - tp)) && c.is_releasable(duel)
}

fn release_group(duel: &Duel, tp: u8, c: CardId) -> CardSet {
    duel.get_matching_group(
        |tc, duel| spconfilter(tc, duel, tp),
        tp,
        LOCATION_HAND,
        LOCATION_MZONE,
        Some(c.into()),
    )
}

fn controler_check() -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    aux::dpcheck(|c, duel| i64::from(c.get_controler(duel)))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let g = release_group(duel, tp, c);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && !g.is_empty() && aux::select_unselect_group_check(duel, g, 2, 2, &controler_check())
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    let rg = release_group(duel, tp, c);
    let g = aux::select_unselect_group_full(duel, rg, 2, 2, &controler_check(), tp, HINTMSG_RELEASE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(g) = e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST)?;
    let Some(c) = a.c else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_REPTILE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_target_card(duel, |_, _, _| true);
    e2.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_monster_effect(duel) && !re.get_handler(duel).is_race(duel, RACE_REPTILE)),
    );
    duel.register_effect(e2, tp);
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
            tp,
            LOCATION_HAND,
            0,
            1,
            None,
        ) && c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)? > 0 && c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    }
    Ok(())
}
