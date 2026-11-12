// Mental Seeker
use crate::prelude::*;
use crate::testing::{HAND, REMOVED};

const ID: u32 = 36565699;

// Banished copies for D.D.R. - Different Dimension Reincarnation in the hand to Special Summon, discarding the other hand cards.
pub const SUPPORT: &[(u32, u16)] = &[(ID, REMOVED), (ID, REMOVED), (9622164, HAND), (9622164, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_REMOVED));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.confirm_decktop(1 - tp, 3)?;
    let g = duel.get_decktop_group(1 - tp, 3);
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg = g.filter_select(duel, tp, CardId::is_able_to_remove, 1, 1, None)?;
        duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
        duel.shuffle_deck(1 - tp)?;
    }
    Ok(())
}
