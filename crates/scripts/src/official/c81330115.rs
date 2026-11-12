// Number 30: Acid Golem of Destruction
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 81330115;
pub const XYZ_NUMBER: u32 = 30;

// In the Extra Deck, Xyz Summoned with Level 3 Normal Monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (32864, MZONE), (1761063, MZONE), (2468169, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) == 0);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 0);
    c.register_effect(duel, e3);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    if a.e.get_handler(duel).get_overlay_count(duel) == 0 {
        duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, a.tp, 2000);
    }
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT) && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    } else {
        duel.damage(tp, 2000, REASON_EFFECT)?;
    }
    Ok(())
}
