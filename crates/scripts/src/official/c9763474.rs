// Haggard Lizardose
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 9763474;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == sg.len()
    }
}

// Sangan and Formula Synchron, two differently named Link materials.
pub const SUPPORT: &[(u32, u16)] = &[(26202165, MZONE), (50091196, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, atkcost);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn atkfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && !c.is_attack(duel, &[atk])
}

fn atkcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_text_attack(duel).max(0);
    c.is_faceup(duel)
        && c.is_attack_below(duel, 2000)
        && c.is_able_to_remove_as_cost(duel)
        && (!c.is_original_race(duel, RACE_REPTILE) || duel.is_player_can_draw(tp, 1))
        && duel.is_existing_target(
            |tc, duel| atkfilter(tc, duel, atk),
            tp,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            Some(c.into()),
        )
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| atkcostfilter(c, duel, tp), tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| atkcostfilter(c, duel, tp), tp, locations, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(false) };
    e.set_labels(duel, &[i64::from(tc.get_text_attack(duel)), tc.get_original_race(duel) as i64]);
    duel.remove(tc, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn labels(duel: &Duel, e: EffectId) -> (i32, u64) {
    let labels = e.get_labels(duel);
    (
        labels.first().copied().unwrap_or(0) as i32,
        labels.get(1).copied().unwrap_or(0) as u64,
    )
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (atk, race) = labels(duel, e);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && atkfilter(chkc, duel, atk));
    }
    if chk == 0 {
        return Ok(true);
    }
    let atk = atk.max(0);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    duel.select_target(
        tp,
        |c, duel| atkfilter(c, duel, atk),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    if race & RACE_REPTILE == 0 {
        e.set_category(duel, CATEGORY_ATKCHANGE);
    } else {
        e.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DRAW);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let (atk, race) = labels(duel, e);
    let atk = atk.max(0);
    if tc.is_attack(duel, &[atk]) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_value(duel, atk);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    if tc.is_attack(duel, &[atk]) && race & RACE_REPTILE == RACE_REPTILE {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
