// White Knight of Dogmatika
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 40352445;

pub const LISTED_NAMES: &[u32] = &[31002402];
// On the field with monsters in its own Extra Deck to send when the opponent activates an effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (2322421, EXTRA), (23693634, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
    c.register_effect(duel, e1);
    aux::add_continuous_lizard_check(duel, c, LOCATION_MZONE, |_, _, _| true);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_ATKCHANGE);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_EXTRA, 0, 1, None)
                && duel.is_existing_matching_card(CardId::is_able_to_grave, tp, 0, LOCATION_EXTRA, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_ALL, LOCATION_EXTRA as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_grave, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(first) = g.get_first() else {
        return Ok(());
    };
    if duel.send_to_grave(g, REASON_EFFECT)? > 0 && first.is_location(duel, LOCATION_GRAVE) {
        let atk1 = first.get_attack(duel);
        let rg = duel.get_matching_group(CardId::is_able_to_grave, tp, 0, LOCATION_EXTRA, None);
        let mut atk2 = 0;
        if !rg.is_empty() {
            let extra = duel.get_field_group(tp, 0, LOCATION_EXTRA);
            duel.confirm_cards(tp, extra)?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let sg = rg.select(duel, tp, 1, 1, None)?;
            duel.send_to_grave(sg, REASON_EFFECT)?;
            duel.shuffle_extra(1 - tp)?;
            if let Some(sc) = sg.get_first()
                && sc.is_location(duel, LOCATION_GRAVE)
            {
                atk2 = sc.get_attack(duel);
            }
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        // The core rounds the Lua float.
        e1.set_value(duel, (f64::from(atk1 + atk2) / 2.0).round() as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
