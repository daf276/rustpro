// Flash Charge Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 95372220;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from Dragons, with Baby Dragons to Summon into its linked zones and the opponent's Mage Power and Mind Control to
// target it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (88819587, MZONE),
    (88819587, MZONE),
    (88819587, MZONE),
    (88819587, HAND),
    (88819587, HAND),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_FORCE_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(
        duel,
        Value::Zones(|duel, e, _, _, _| !(e.get_handler(duel).get_linked_zone(duel) as i32)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_NEGATE);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e4.set_condition(duel, negcon);
    e4.set_cost(duel, negcost);
    e4.set_target(duel, negtg);
    e4.set_operation(duel, negop);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    if c.is_location(duel, LOCATION_MZONE) {
        ec.get_linked_group(duel).contains(c)
    } else {
        ec.get_linked_zone_full(duel, c.get_previous_controler(duel)) & (1 << c.get_previous_sequence(duel)) != 0
    }
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let ec = a.e.get_handler(duel);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, ec), 1, None))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, g) = (a.tp, a.e.get_handler(duel).get_linked_group(duel));
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = a.e.get_handler(duel).get_linked_group(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let tg = g.select(duel, tp, 1, 1, None)?;
    if !tg.is_empty() {
        duel.hint_selection(tg);
        if duel.destroy(tg, REASON_EFFECT)? > 0 {
            duel.damage(1 - tp, 500, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.contains(c)) && duel.is_chain_negatable(a.ev as u8)
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_full(tp, costfilter, 1, ReleaseOptions::new(tp)));
    }
    let g = duel.select_release_group_full(tp, costfilter, (1, 1), false, ReleaseOptions::new(tp))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
