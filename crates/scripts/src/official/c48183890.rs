// Traptrix Atypus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 48183890;

pub const LISTED_SERIES: &[u16] = &[SET_TRAPTRIX];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_race_full(duel, RACE_INSECT | RACE_PLANT, Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// In the Extra Deck, Link Summoned with Traptrix Atrax (Insect); Trap Hole (a Normal Trap) in the Graveyard and monsters of the
// opponent to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (55428242, MZONE),
    (55428242, MZONE),
    (55428242, MZONE),
    (4206964, GRAVE),
    (4206964, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_value(duel, Value::Effect(|duel, _, te| te.is_trap_effect(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, atkcon);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_TRAPTRIX]));
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE + CATEGORY_REMOVE + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn ntfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(ntfilter, a.e.get_handler_player(duel), LOCATION_GRAVE, 0, 1, None)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_negatable(duel));
    }
    let ct = duel.get_matching_group_count(
        aux::faceup_filter(|c, duel| c.is_race(duel, RACE_INSECT | RACE_PLANT)),
        tp,
        LOCATION_MZONE,
        0,
        None,
    );
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, ct as u8, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if g.is_empty() {
        return Ok(());
    }
    let mut neg_chk = false;
    let c = e.get_handler(duel);
    for tc in g {
        if tc.is_can_be_disabled_by_effect(duel, e, true) {
            neg_chk = true;
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        tc.register_effect(duel, e2);
    }
    if !neg_chk {
        return Ok(());
    }
    let rg = duel.get_matching_group(
        |c, duel| ntfilter(c, duel) && c.is_able_to_remove(duel),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if rg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let srg = rg.select(duel, tp, 1, 1, None)?;
    if srg.is_empty() {
        return Ok(());
    }
    duel.break_effect()?;
    if duel.remove(srg, POS_FACEUP, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = g.select(duel, tp, 1, 1, None)?;
    if !dg.is_empty() {
        duel.hint_selection_full(dg, true);
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
