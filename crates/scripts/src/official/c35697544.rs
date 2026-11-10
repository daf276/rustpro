// Fighting Flame Sword
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 35697544;
pub const LISTED_NAMES: &[u32] = &[CARD_FLAME_SWORDSMAN, ID];
// The fields of its chain data table.
const CHOICE: u8 = 0;
const NEGATE_CHAIN_LINK: u8 = 1;

// In the hand, to search Fighting Flame Swordsman.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (1047075, DECK), (1047075, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_FLAME_SWORDSMAN) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn disconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    ((c.is_code(duel, CARD_FLAME_SWORDSMAN) && c.is_on_field(duel))
        || (c.lists_code(duel, CARD_FLAME_SWORDSMAN) && c.is_location(duel, LOCATION_MZONE)))
        && c.is_faceup(duel)
        && c.is_controler(duel, tp)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(e.get_chain_data(duel, CHOICE) == Some(2) && chkc.is_on_field(duel) && chkc != c);
    }
    let bc = duel.get_battle_monster(tp).0;
    let event = duel.check_event_full(EVENT_CHAINING);
    let tg = event.and_then(|event| duel.get_chain_info::<Option<CardSet>>(event.event_value as u8, CHAININFO_TARGET_CARDS));
    let b1 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.check_event(EVENT_ATTACK_ANNOUNCE)
        && bc.is_some_and(|bc| bc.is_attribute(duel, ATTRIBUTE_FIRE) && bc.is_race(duel, RACE_WARRIOR) && bc.is_faceup(duel))
        && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()));
    let b3 = event.is_some_and(|event| {
        event.event_player == 1 - tp
            && event
                .reason_effect
                .is_some_and(|re| re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET))
            && tg.is_some_and(|tg| tg.is_exists(duel, |c, duel| disconfilter(c, duel, tp), 1, None))
            && duel.is_chain_disablable(event.event_value as u8)
    });
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let choice = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    match choice {
        Some(1) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            e.set_property(duel, 0);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_DESTROY);
            e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
            duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
        }
        Some(3) => {
            if let Some(event) = event {
                e.set_chain_data(duel, NEGATE_CHAIN_LINK, i64::from(event.event_value));
                e.set_category(duel, CATEGORY_DISABLE);
                e.set_property(duel, 0);
                let rc = event.reason_effect.map(|re| re.get_handler(duel).into());
                duel.set_operation_info(0, CATEGORY_DISABLE, rc, 1, tp, 0);
            }
        }
        _ => {}
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        Some(2) => {
            if let Some(tc) = duel.get_first_target()
                && tc.is_relate_to_effect(duel, e)
            {
                duel.destroy(tc, REASON_EFFECT)?;
            }
        }
        Some(3) => {
            if let Some(link) = e.get_chain_data(duel, NEGATE_CHAIN_LINK) {
                duel.negate_effect(link as u8);
            }
        }
        _ => {}
    }
    Ok(())
}
