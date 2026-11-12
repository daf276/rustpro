// Bettan Bat
// s.toss_coin, which only cards that are not ported read, is left out.
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 84079032;

// On the field, with the opponent's Sangan Summoned, flipped and attacked into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COIN + CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit_full(duel, 3, 0, EFFECT_COUNT_CODE_SINGLE);
    e1.set_condition(duel, |duel, a| {
        let eg = a.eg.unwrap_or_default();
        !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - a.tp), 1, None)
    });
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(
        duel,
        EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP,
        EFFECT_FLAG2_CHECK_SIMULTANEOUS,
    );
    e3.set_code(duel, EVENT_CHANGE_POS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, postg2);
    e3.set_operation(duel, posop2);
    c.register_effect(duel, e3);
}

fn posfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_can_turn_set(duel) || (!c.is_attack_pos(duel) && c.is_can_change_position(duel)))
        && c.is_can_be_effect_target(duel, e)
        && c.is_summon_player(duel, 1 - tp)
        && c.is_location(duel, LOCATION_MZONE)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp, eg) = (a.e, a.tp, a.eg.unwrap_or_default());
    if let Some(chkc) = chkc {
        return Ok(eg.is_contains(chkc) && posfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| posfilter(c, duel, e, tp), 1, None));
    }
    let g = eg.filter(duel, |c, duel| posfilter(c, duel, e, tp), None);
    let mut tg = g.get_first().map(CardSet::from).unwrap_or_default();
    if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        tg = g.select(duel, tp, 1, 1, None)?;
    }
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, Some(tg), 1, tp, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let coin = duel.toss_coin(tp, 1)?.first().copied().map_or(0, u32::from);
    duel.change_position(
        tc,
        if coin == COIN_HEADS {
            POS_FACEUP_ATTACK
        } else {
            POS_FACEDOWN_DEFENSE
        },
    )?;
    Ok(())
}

fn posfilter2(c: CardId, duel: &Duel, e: EffectId) -> bool {
    (c.is_can_turn_set(duel) || !c.is_position(duel, POS_FACEUP_ATTACK))
        && c.is_can_be_effect_target(duel, e)
        && c.is_can_change_position(duel)
        && ((c.is_position(duel, POS_FACEUP) && c.is_previous_position(duel, POS_FACEDOWN))
            || (c.is_position(duel, POS_FACEDOWN_DEFENSE) && c.is_previous_position(duel, POS_FACEUP)))
}

fn postg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp, eg) = (a.e, a.tp, a.eg.unwrap_or_default());
    if let Some(chkc) = chkc {
        return Ok(eg.is_contains(chkc) && chkc.is_location(duel, LOCATION_MZONE) && posfilter2(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| posfilter2(c, duel, e), 1, None));
    }
    let g = eg.filter(duel, |c, duel| posfilter2(c, duel, e), None);
    let mut tg = g.get_first().map(CardSet::from).unwrap_or_default();
    if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        tg = g.select(duel, tp, 1, 1, None)?;
    }
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(tg), 1, tp, 0);
    Ok(false)
}

fn posop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let opt = if tc.is_position(duel, POS_FACEDOWN) || tc.is_type(duel, TYPE_TOKEN) {
        POS_FACEUP_ATTACK
    } else if tc.is_position(duel, POS_FACEUP_ATTACK) {
        POS_FACEDOWN_DEFENSE
    } else if tc.is_position(duel, POS_FACEUP_DEFENSE) {
        POS_FACEUP_ATTACK | POS_FACEDOWN_DEFENSE
    } else {
        0
    };
    if opt == 0 {
        return Ok(());
    }
    let pos = duel.select_position(tp, tc, opt as u8)?;
    if pos == 0 {
        return Ok(());
    }
    duel.change_position(tc, pos)?;
    Ok(())
}
