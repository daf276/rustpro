// Ripple Bird
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 56410769;

// Twice in the Extra Deck, Xyz Summoned with Level 1 Normal Monsters (Skull Servant, Bone Mouse, Protron).
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (32274490, MZONE), (21239280, MZONE), (24154052, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(1);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, |duel, a| poscon(duel, a, POS_ATTACK));
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| poscon(duel, a, POS_DEFENSE));
    c.register_effect(duel, e3);
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_can_change_position(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}

fn poscon(duel: &Duel, a: &Args, pos: u32) -> bool {
    let g = duel.get_field_group(a.e.get_handler_player(duel), LOCATION_MZONE, 0);
    g.len() == g.filter_count(duel, |c, duel| c.is_position(duel, pos), None)
}
