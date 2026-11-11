// Jurrac Gallim
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 43332022;

// On the field, destroyed in battle by the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE)
    });
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tc = a.e.get_handler(duel).get_reason_card(duel);
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_battle(duel)
        && !duel.is_existing_matching_card(CardId::is_discardable, 1 - a.tp, LOCATION_HAND, 0, 1, None)
    {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, 0, 0);
    }
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tc = a.e.get_handler(duel).get_reason_card(duel);
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !tc.is_relate_to_battle(duel) {
        return Ok(());
    }
    if duel.is_existing_matching_card(CardId::is_discardable, 1 - tp, LOCATION_HAND, 0, 1, None)
        && duel.select_yes_no(1 - tp, aux::stringid(ID, 1))?
    {
        duel.discard_hand(1 - tp, CardId::is_discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    } else {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
