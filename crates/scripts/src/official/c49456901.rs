// Number C104: Umbral Horror Masquerade
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 49456901;

pub const LISTED_NAMES: &[u32] = &[2061963];

// In the Extra Deck thrice, Xyz Summoned with Orion the Battle King, Shapesnatch, Queen of Autumn Leaves and
// Leo Wizard, and Spell/Trap Cards to destroy. The negation needs Number 104: Masquerade as material, which
// nothing here attaches.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (2971090, MZONE),
    (4035199, MZONE),
    (4179849, MZONE),
    (4392470, MZONE),
    (2971090, MZONE),
    (4179849, MZONE),
    (5318639, SZONE),
    (5318639, SZONE | OPPONENT),
    (44095762, SZONE),
    (4206964, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 4;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, a.e)) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let triggering_location: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    c.get_overlay_group(duel)
        .is_exists(duel, |mc, duel| mc.is_code(duel, 2061963), 1, None)
        && a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && triggering_location == LOCATION_MZONE
        && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(a.ev as u8)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.negate_activation(a.ev as u8) {
        return Ok(());
    }
    let opp = 1 - tp;
    let g = duel.get_field_group(opp, LOCATION_HAND, 0);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    let Some(sc) = g.random_select(duel, opp, 1).get_first() else {
        return Ok(());
    };
    duel.break_effect()?;
    if duel.send_to_grave(sc, REASON_EFFECT)? > 0 && sc.is_location(duel, LOCATION_GRAVE) {
        // Lua halves with a float, which the core rounds.
        duel.set_lp(opp, (f64::from(duel.get_lp(opp)) / 2.0).round() as i32);
    }
    Ok(())
}
