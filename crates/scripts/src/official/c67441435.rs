// Glow-Up Bulb
use crate::prelude::*;
use crate::testing::GRAVE;

const ID: u32 = 67441435;

// Copies in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.discard_deck(tp, 1, REASON_EFFECT)? != 0 {
        let oc = duel.get_operated_group().get_first();
        let c = e.get_handler(duel);
        if oc.is_some_and(|oc| oc.is_location(duel, LOCATION_GRAVE)) && c.is_relate_to_effect(duel, e) {
            duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
