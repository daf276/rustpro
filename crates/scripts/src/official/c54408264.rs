// Magikey Battle
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 54408264;

pub const LISTED_SERIES: &[u16] = &[SET_MAGIKEY];

const MAGIKEY_MAFTEA: u32 = 99426088;

pub const LISTED_NAMES: &[u32] = &[MAGIKEY_MAFTEA];

// In hand, with Bolt Penguins (Normal Monsters) in the Graveyard to shuffle into the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (48531733, GRAVE), (48531733, GRAVE), (48531733, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    ((c.is_monster(duel) && (c.is_type(duel, TYPE_NORMAL) || c.is_set_card(duel, &[SET_MAGIKEY]))) || c.is_code(duel, MAGIKEY_MAFTEA))
        && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let ch = i32::from(duel.get_current_chain_full(true)) - 1;
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && ch > 0
        && duel.get_chain_info::<u8>(ch as u8, CHAININFO_TRIGGERING_PLAYER) != tp
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_target_card(duel, |duel, _, c| {
            c.is_set_card(duel, &[SET_MAGIKEY]) || (c.is_type(duel, TYPE_NORMAL) && !c.is_type(duel, TYPE_TOKEN))
        });
        e1.set_value(duel, Value::Effect(|duel, e, re| e.get_label_object(duel) == Some(re)));
        e1.set_label_object(duel, duel.get_chain_info::<Option<EffectId>>(ch as u8, CHAININFO_TRIGGERING_EFFECT));
        e1.set_reset(duel, RESET_CHAIN);
        duel.register_effect(e1, tp);
    }
    Ok(())
}
