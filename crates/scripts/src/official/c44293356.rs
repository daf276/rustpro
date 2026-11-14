// Megalith Notrah Plura
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 44293356;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];
pub const MAT_FILTER: fn(CardId, &Duel, u8) -> bool = |c, duel, _| c.is_ritual_monster(duel);

// In the hand to reveal and to be Ritual Summoned by Megalith Hagith with Megalith Bethor (Ritual Monsters, Level 4 and 8) but not
// Black Luster Soldier (a Level 8 Normal Monster), and on the field to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (90444325, MZONE),
    (90444325, MZONE),
    (99628747, HAND),
    (99628747, MZONE),
    (10000100, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_ritual_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !duel.has_flag_effect(a.tp, ID));
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_RELEASE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.ep == 1 - a.tp && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
    });
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let c = a.e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_operation(duel, inactop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn inactop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    if a.ep == a.tp && chain::is_setcode(duel, ev, &[SET_MEGALITH]) && chain::is_composite_type(duel, ev, TYPE_RITUAL | TYPE_MONSTER) {
        duel.set_chain_limit(|_, _, rp, tp, _| rp == tp, 0);
    }
    Ok(())
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
        }
    }
    let tg = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS);
    if tg.is_some_and(|tg| tg.is_exists(duel, CardId::is_on_field, 1, None)) {
        e.set_label(duel, 100);
        duel.set_possible_operation_info(0, CATEGORY_RELEASE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    } else {
        e.set_label(duel, 0);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.negate_activation(a.ev as u8) {
        return Ok(());
    }
    if let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    if e.get_label(duel) == 100
        && duel.is_existing_matching_card(CardId::is_releasable_by_effect, tp, 0, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
        let g = duel.select_matching_card(tp, CardId::is_releasable_by_effect, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.break_effect()?;
            duel.release(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
