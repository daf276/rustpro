// Neo-Spacian Aqua Dolphin
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 17955766;

// On the field, with cards in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (26202165, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::discard);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(tp, g)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_OPPO);
    if let Some(sc) = g.filter_select(duel, tp, CardId::is_monster, 1, 1, None)?.get_first() {
        let atk = sc.get_attack(duel);
        let stronger = aux::faceup_filter(move |c, duel| c.is_attack_above(duel, atk));
        if sc.is_attack_above(duel, 0) && duel.is_existing_matching_card(stronger, tp, LOCATION_MZONE, 0, 1, None) {
            if duel.destroy(sc, REASON_EFFECT)? > 0 {
                duel.damage(1 - tp, 500, REASON_EFFECT)?;
            }
        } else {
            duel.damage(tp, 500, REASON_EFFECT)?;
        }
    }
    duel.shuffle_hand(1 - tp)?;
    Ok(())
}
