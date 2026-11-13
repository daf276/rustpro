// Perfectron Hydradrive Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 13203964;

// In the Extra Deck (Link-5), Link Summoned from Firewall Dragon (Link-4) and Link Spider (Link-1) with another in the Graveyard, against
// the opponent's monsters; the destruction replacement is no chain link and moves no card, so its prompt is the MSG_SELECT_EFFECTYN.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (5043010, MZONE),
    (5043010, MZONE),
    (98978921, MZONE),
    (98978921, MZONE),
    (98978921, GRAVE),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_LINK, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1a.set_target(duel, destg);
    e1a.set_operation(duel, desop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_DAMAGE_STEP_END);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_ADD_ATTRIBUTE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, (ATTRIBUTE_LIGHT | ATTRIBUTE_WATER | ATTRIBUTE_FIRE | ATTRIBUTE_WIND) as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, desreptg);
    c.register_effect(duel, e4);
}

fn linkdamage(duel: &Duel, tp: u8) -> i32 {
    300 * duel.get_matching_group_count(CardId::is_link_monster, tp, LOCATION_GRAVE, 0, None) as i32
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
    let dam = linkdamage(duel, tp);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
    if !g.is_empty() && duel.destroy(g, REASON_EFFECT)? > 0 {
        let dam = linkdamage(duel, tp);
        if dam > 0 {
            duel.break_effect()?;
            duel.damage(1 - tp, dam as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn immval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    let trig_loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_LOCATION);
    let trig_attr: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_ATTRIBUTE);
    if !(te.is_monster_effect(duel) && te.is_activated(duel)) {
        return false;
    }
    let c = e.get_handler(duel);
    let tc = te.get_handler(duel);
    if !duel.is_chain_solving() || (tc.is_relate_to_effect(duel, te) && tc.is_faceup(duel) && tc.is_location(duel, trig_loc)) {
        c.is_attribute(duel, tc.get_attribute(duel))
    } else {
        c.get_attribute(duel) & trig_attr > 0
    }
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_attack_above(duel, 1000) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT) && !c.is_reason(duel, REASON_REPLACE));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        let atk = c.get_attack(duel) - 1000;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        return Ok(true);
    }
    Ok(false)
}
