// Dried Winds
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 28265983;

pub const LISTED_SERIES: &[u16] = &[SET_AROMA];
// In the hand and the Spell & Trap Zone, with Aromaseraphy Angelica and Aromalilith Rosalina (Aroma monsters that gain LP) and
// Dian Keto the Cure Master and Goblin's Secret Remedy in the hand, against the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (16759958, MZONE),
    (24689197, MZONE),
    (84257639, HAND),
    (11868825, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 4)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_code(duel, EVENT_RECOVER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |_, a| a.ep == a.tp);
    e1.set_target(duel, destg1);
    e1.set_operation(duel, desop1);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 4));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, descon2);
    e3.set_cost(duel, descost2);
    e3.set_target(duel, destg2);
    e3.set_operation(duel, desop2);
    c.register_effect(duel, e3);
}

fn destg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop1(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon2(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_AROMA])),
        tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    ) && duel.get_lp(tp) - duel.get_lp(1 - tp) >= 3000
}

fn descost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let lp = duel.get_lp(tp) - duel.get_lp(1 - tp);
    if chk == 0 {
        unported!(duel, duel.is_player_affected_by_effect(tp, EFFECT_LPCOST_CHANGE).is_some());
        return Ok(duel.check_lp_cost(tp, lp as u32));
    }
    duel.pay_lp_cost(tp, lp as u32)?;
    a.e.set_label(duel, i64::from(lp));
    Ok(false)
}

fn desfilter2(c: CardId, duel: &Duel, num: i32) -> bool {
    c.is_faceup(duel) && c.is_attack_below(duel, num)
}

fn destg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lp = duel.get_lp(tp) - duel.get_lp(1 - tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| desfilter2(c, duel, lp), tp, 0, LOCATION_MZONE, 1, None));
    }
    let num = a.e.get_label(duel) as i32;
    let g = duel.get_matching_group(|c, duel| desfilter2(c, duel, num), tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet, lp: i32) -> (bool, bool) {
    (sg.get_sum(duel, |c, duel| i64::from(c.get_attack(duel))) <= i64::from(lp), false)
}

fn desop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let num = a.e.get_label(duel) as i32;
    let g = duel.get_matching_group(|c, duel| desfilter2(c, duel, num), tp, 0, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let dg = aux::select_unselect_group(duel, g, 1, g.len(), &|duel, sg, _, _| rescon(duel, sg, num), tp, HINTMSG_DESTROY)?;
    duel.destroy(dg, REASON_EFFECT)?;
    Ok(())
}
