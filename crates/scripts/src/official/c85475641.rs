// Infernity Avenger
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 85475641;

// In the Graveyard with Kojikocy destroyed by battle (Elfnote Lucina attacks it) once the hand has been played out.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (1184620, MZONE),
    (1184620, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition_mut(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_GRAVE)
        && c.is_reason(duel, REASON_BATTLE)
        && c.has_level(duel)
        && c.is_controler(duel, tp)
        && c.is_previous_controler(duel, tp)
        && c.get_code(duel) != ID
}

fn spcon(duel: &mut Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| filter(c, duel, tp), None);
    let Some(tc) = g.get_first() else { return false };
    e.set_label(duel, i64::from(tc.get_level(duel)));
    duel.get_field_group_count(tp, LOCATION_HAND, 0) == 0
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if duel.get_field_group_count(tp, LOCATION_HAND, 0) != 0 || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_LEVEL);
    e1.set_value(duel, e.get_label(duel) as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    duel.special_summon_complete()?;
    Ok(())
}
