// Fabled Andwraith
use super::c54048462::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 9061682;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Synchro Summoned with The Fabled Chawa (a Level 1 Tuner) and Shapesnatch (Level 5), with monsters in the opponent's hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (29905795, MZONE),
    (29905795, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (3573512, HAND | OPPONENT),
    (3573512, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2));
    }
    duel.set_target_player(tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let hg = duel.get_field_group(1 - tp, LOCATION_HAND, 0);
    if !hg.is_empty() && duel.select_yes_no(1 - tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, 1 - tp, aux::stringid(ID, 3));
        duel.discard_hand(1 - tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
        if duel.is_chain_disablable(0) {
            duel.negate_effect(0);
            return Ok(());
        }
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    if duel.draw(p, 2, REASON_EFFECT)? == 2 {
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel) && c.get_previous_controler(duel) == 1 - tp && c.is_previous_location(duel, LOCATION_HAND)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && cfilter(c, duel, tp)
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_GRAVE)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && !g.is_empty());
    }
    let c = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        g.select(duel, tp, 1, 1, None)?.get_first()
    } else {
        g.get_first()
    };
    if let Some(c) = c {
        duel.set_target_card(c);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, c.map(Into::into), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?
    {
        let handler = e.get_handler(duel);
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(handler);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
