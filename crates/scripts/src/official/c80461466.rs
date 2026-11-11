// Great Mammoth of the Netherworld
use crate::prelude::*;
use crate::testing::{Check, GRAVE, SZONE};

const ID: u32 = 80461466;

pub const LISTED_NAMES: &[u32] = &[CARD_CALL_OF_THE_HAUNTED, ID];
// In the Graveyard with Call of the Haunted set to Special Summon it, which also satisfies the condition.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (CARD_CALL_OF_THE_HAUNTED, SZONE),
    (CARD_CALL_OF_THE_HAUNTED, SZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_XMATERIAL);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_race(duel, RACE_ZOMBIE) && c.is_xyz_monster(duel)
    });
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
}

fn desconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && (c.is_code(duel, CARD_CALL_OF_THE_HAUNTED)
            || (c.is_location(duel, LOCATION_MZONE) && c.is_race(duel, RACE_ZOMBIE) && !c.is_code(duel, ID)))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
            && duel.is_existing_matching_card(desconfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}
