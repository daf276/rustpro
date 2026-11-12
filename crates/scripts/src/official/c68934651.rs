// Firewall Dragon Darkfluid
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 68934651;
const COUNTER_FW: u16 = 0x14c;
const TYPES: u32 = TYPE_FUSION + TYPE_RITUAL + TYPE_SYNCHRO + TYPE_XYZ;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with three Sangans (Effect Monsters) over Cyberse Integrator (Synchro) and Cyberse Reminder (Xyz)
// in the Graveyard, with the opponent's monsters to negate and to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (25200959, GRAVE),
    (56337500, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.enable_counter_permit(duel, COUNTER_FW);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, cttg);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| duel.is_battle_phase());
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(c.get_counter(duel, COUNTER_FW)) * 2500));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_NEGATE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, discon);
    e3.set_cost(duel, discost);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPES) && c.is_race(duel, RACE_CYBERSE)
}

fn getcount(duel: &Duel, tp: u8) -> u32 {
    let g = duel.get_matching_group(ctfilter, tp, LOCATION_GRAVE, 0, None);
    (g.iter().fold(0, |tottype, c| tottype | c.get_type(duel)) & TYPES).count_ones()
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let ct = getcount(duel, a.tp);
    if chk == 0 {
        return Ok(ct > 0 && a.e.get_handler(duel).is_can_add_counter(duel, COUNTER_FW, ct as u16));
    }
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) || c.is_facedown(duel) {
        return Ok(());
    }
    let ct = getcount(duel, a.tp) as u16;
    c.add_counter(duel, COUNTER_FW, ct);
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && duel.is_chain_negatable(a.ev as u8)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_remove_counter(duel, tp, COUNTER_FW, 1, REASON_COST));
    }
    c.remove_counter(duel, tp, COUNTER_FW, 1, REASON_COST)?;
    Ok(false)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let tc = duel.get_attacker();
    if duel.negate_activation(a.ev as u8) && Some(c) == tc && c.can_chain_attack_full(duel, 0, false) && c.is_relate_to_effect(duel, e) {
        duel.chain_attack();
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_DAMAGE_STEP_END);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_operation(duel, |duel, _| {
            duel.chain_attack();
            Ok(())
        });
        e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
