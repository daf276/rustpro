// Symphonic Warrior Piaano
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 31826057;

// Copies on the field to change the Type of and in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE), (ID, GRAVE), (ID, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target::<{ LOCATION_MZONE }>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, target::<0>);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SYMPHONIC_WARRIOR])
}

fn target<const OPPO: u32>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && (OPPO == 0 || chkc.is_controler(duel, tp))
            && filter(chkc, duel)
            && chkc.is_race_except(duel, e.get_label(duel) as u64));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, OPPO, 1, None));
    }
    let g = duel.get_matching_group(
        |c, duel| filter(c, duel) && c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_MZONE,
        OPPO,
        None,
    );
    let rc = duel.announce_another_race(g, tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let sel = g.filter_select(duel, tp, |c, duel| c.is_race_except(duel, rc), 1, 1, None)?;
    duel.set_target_card(sel);
    e.set_label(duel, rc as i64);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_RACE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, e.get_label(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
