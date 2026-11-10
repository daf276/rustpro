// Alpha, the Master of Beasts
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 73304257;

// In the hand to Special Summon against the opponent's stronger board (Sangan), with Beasts (Ancient One of the Deep Forest) on
// the field to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (14015067, MZONE),
    (14015067, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn sumatk(duel: &Duel, tp: u8) -> i64 {
    duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, 0, None)
        .get_sum(duel, |c, duel| i64::from(c.get_attack(duel)))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && sumatk(duel, 1 - tp) > sumatk(duel, tp)
}

fn sthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB) && c.is_able_to_hand(duel)
}

fn othfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && sthfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(sthfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_matching_card(othfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let ct = duel.get_matching_group_count(othfilter, tp, 0, LOCATION_MZONE, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let tg = duel.select_target(tp, sthfilter, tp, LOCATION_MZONE, 0, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, tg, ct as u8 * 2, PLAYER_ALL, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_target_card(duel, |duel, _, c| c.is_code(duel, ID));
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)? > 0 {
        let og = duel.get_operated_group();
        let ct = og.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
        let g = duel.get_matching_group(othfilter, tp, 0, LOCATION_MZONE, None);
        if ct == 0 || g.len() < ct {
            return Ok(());
        }
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let sg = g.select(duel, tp, ct as u8, ct as u8, None)?;
        duel.hint_selection(sg);
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
