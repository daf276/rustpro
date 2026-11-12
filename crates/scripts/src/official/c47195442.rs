// Wonky Quartet
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 47195442;

// In the Extra Deck, Xyz Summoned with Level 4 Normal Monsters (Prevent Rat, Kojikocy, Uraby).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (1784619, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_CONTROL + CATEGORY_DAMAGE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e2.set_target(duel, atchtg);
    e2.set_operation(duel, atchop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    c.register_effect(duel, e3);
}

fn atchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_XYZ) && duel.get_field_group_count(tp, LOCATION_DECK, 0) >= 3);
    }
    duel.set_possible_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, 1 - tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 1, tp, 400);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn atchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let g = duel.get_decktop_group(tp, 3);
    if !(c.is_relate_to_effect(duel, e) && g.len() == 3) {
        return Ok(());
    }
    duel.disable_shuffle_check(true);
    duel.overlay(c, g)?;
    let og = c.get_overlay_group(duel);
    duel.break_effect()?;
    if og.filter_count(duel, CardId::is_monster, None) <= 4 {
        duel.get_control(c, 1 - tp)?;
    } else if duel.damage(tp, og.len() as u32 * 400, REASON_EFFECT)? > 0 {
        let dg = duel.get_field_group(tp, LOCATION_MZONE, 0);
        if !dg.is_empty() {
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
