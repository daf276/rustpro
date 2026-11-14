// Dragunity Knight - Areadbhair
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 88234821;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGUNITY];
// Two copies on the field (each may banish itself for its negation) to battle Mystical Elf of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (15025844, MZONE | OPPONENT), (15025844, MZONE | OPPONENT)];
// The banish, whose target rmcon keeps as the label object.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, _sc: CardId, _sumtype: u64, _tp: u8) -> bool {
        c.is_set_card(duel, &[SET_DRAGUNITY])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, negcost);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition_mut(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.re.is_some_and(|re| re.is_monster_effect(duel))
        && a.rp == 1 - a.tp
        && !c.is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(a.ev as u8)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DRAGUNITY])
        && c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, cfilter, LOCATION_GRAVE | LOCATION_MZONE)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    if chk == 0 {
        return Ok(
            rc.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT) || (!relation && duel.is_player_can_remove(tp, None))
        );
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if relation {
        duel.set_operation_info(
            0,
            CATEGORY_REMOVE,
            Some(rc.into()),
            1,
            rc.get_controler(duel),
            rc.get_location(duel) as i32,
        );
    } else {
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, 0, rc.get_previous_location(duel) as i32);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if duel.negate_activation(a.ev as u8) && rc.is_relate_to_effect(duel, re) {
        duel.remove(rc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn rmcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    match c.get_battle_target(duel) {
        Some(bc) if bc.is_controler(duel, 1 - a.tp) && bc.is_status(duel, STATUS_BATTLE_DESTROYED) => {
            a.e.set_label_object_group(duel, Some(bc.into()));
            true
        }
        _ => false,
    }
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let bc = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_REMOVE, bc, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(bc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if bc.is_relate_to_battle(duel) && bc.is_controler(duel, 1 - a.tp) {
        duel.remove(bc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_synchro_summoned(duel) && a.rp == 1 - a.tp && c.is_previous_controler(duel, a.tp)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let sg = duel.get_matching_group(|c, duel| c.is_spell_trap(duel), a.tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!sg.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let sg = duel.get_matching_group(|c, duel| c.is_spell_trap(duel), a.tp, 0, LOCATION_ONFIELD, None);
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}
