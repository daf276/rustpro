// Dragunity Knight - Barcha
use super::c21249921::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 25682811;

// In the Extra Deck, Synchro Summoned from Darkspear (a Level 3 Dragon Tuner), Zephyros (a Level 4 Winged Beast) and Tribus (Level 1),
// with Dragunity monsters in the Graveyard to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (13361027, MZONE),
    (13361027, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (81962318, MZONE),
    (81962318, MZONE),
    (99594764, GRAVE),
    (54455664, GRAVE),
    (8039562, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| {
            300 * c
                .get_equip_group(duel)
                .filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_DRAGUNITY]), None) as i32
        }),
    );
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DRAGUNITY]) && c.is_race(duel, RACE_DRAGON) && !c.is_forbidden(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, ft as u8, None)?;
    let count = g.map_or(0, |g| g.len()) as u8;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_EQUIP, g, count, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sg = duel.get_target_cards(e).unwrap_or_default();
    if duel.get_location_count(tp, LOCATION_SZONE) < sg.len() as i32 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    for tc in sg {
        c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, None)?;
    }
    Ok(())
}
