// Battlewasp - Grand Partisan the Revolution
use super::c90587641::Procedure;
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, REMOVED, SZONE};

const ID: u32 = 35057188;

// Copies on the field to attack into the opponent's Dimensional Prison and be banished, with a banished Battlewasp - Pin the
// Bullseye for the destruction.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (70342110, SZONE | OPPONENT),
    (70342110, SZONE | OPPONENT),
    (70342110, SZONE | OPPONENT),
    (65899613, REMOVED),
    (65899613, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_INSECT) && c.is_type(duel, TYPE_SYNCHRO));
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| {
            200 * duel.get_matching_group_count(insect_filter, e.get_handler_player(duel), LOCATION_REMOVED, 0, None) as i32
        }),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_REMOVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn insect_filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_INSECT)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    let ct = duel.get_matching_group_count(insect_filter, tp, LOCATION_REMOVED, 0, None);
    if !g.is_empty() && ct > 0 && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let sg = g.select(duel, tp, 1, ct.min(g.len()) as u8, None)?;
        if sg.is_empty() {
            return Ok(());
        }
        duel.hint_selection(sg);
        duel.break_effect()?;
        let des_ct = duel.destroy(sg, REASON_EFFECT)?;
        if des_ct > 0 {
            duel.break_effect()?;
            duel.damage(1 - tp, des_ct as u32 * 500, REASON_EFFECT)?;
        }
    }
    Ok(())
}
