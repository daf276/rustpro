// Number 59: Crooked Cook
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 82697249;
pub const XYZ_NUMBER: u32 = 59;

// In the Extra Deck, Xyz Summoned with two Level 4 Normal Monsters (Kojikocy), with more cards on the field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_ONFIELD, 0) == 1
    });
    e1.set_value(duel, Value::Effect(|duel, e, te| te.get_owner(duel) != e.get_owner(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, 0, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_GRAVE) && c.is_monster(duel)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_ONFIELD, 0, Some(c.into()));
    if duel.destroy(g, REASON_EFFECT)? != 0 {
        let ct = duel.get_operated_group().filter_count(duel, ctfilter, None);
        if ct > 0 && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
            duel.break_effect()?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
            e1.set_value(duel, ct as i32 * 300);
            e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
