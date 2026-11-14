// Elemental HERO Stratos
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 40044918;

pub const LISTED_SERIES: &[u16] = &[SET_HERO];
// In the hand with Elemental HERO Sparkman (a Normal Monster) on the field and in the Deck, and Spells/Traps on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (20721928, MZONE),
    (20721928, DECK),
    (CARD_MYSTICAL_SPACE_TYPHOON, SZONE),
    (CARD_MYSTICAL_SPACE_TYPHOON, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HERO]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_HERO])),
        tp,
        LOCATION_MZONE,
        0,
        1,
        Some(e.get_handler(duel).into()),
    ) && duel.is_existing_matching_card(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None);
    let b2 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        let g = duel.get_matching_group(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        e.set_category(duel, CATEGORY_DESTROY);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            let c = e.get_handler(duel);
            let exc = c.is_relate_to_effect(duel, e).then_some(c.into());
            let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_HERO]));
            let ct = duel.get_matching_group_count(filter, tp, LOCATION_MZONE, 0, exc);
            if ct == 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, ct as u8, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}
