// CXyz Comics Hero Legend Arthur
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 13030280;

pub const LISTED_NAMES: &[u32] = &[77631175];

// On the field with Comics Hero King Arthur attached by Xyz Import, attacking the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (14602126, HAND),
    (77631175, MZONE | OPPONENT),
    (77631175, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_count_limit(duel, 1, 0);
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_DAMAGE);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_condition(duel, damcon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_code(duel, 77631175), 1, None)
        && c.is_relate_to_battle(duel)
        && c.get_battle_target(duel)
            .is_some_and(|bc| bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel))
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(bc) = a.e.get_handler(duel).get_battle_target(duel) else {
        return Ok(false);
    };
    if chk == 0 {
        return Ok(duel
            .is_player_affected_by_effect(bc.get_controler(duel), CARD_SPIRIT_ELIMINATION)
            .is_none()
            && bc.is_able_to_remove(duel));
    }
    duel.set_target_card(bc);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(bc.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, bc.get_base_attack(duel));
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(bc) = duel.get_first_target()
        && bc.is_relate_to_effect(duel, e)
        && duel.remove(bc, POS_FACEUP, REASON_EFFECT)? > 0
    {
        duel.damage(1 - tp, bc.get_base_attack(duel) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
