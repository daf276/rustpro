// Zubababa Knight
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 62006866;

pub const LISTED_NAMES: &[u32] = &[ID];

// Zubababancho Gagagacoat (Zubaba) and Gagaga Ganbara Knight in the Deck, with copies on the field to be Xyz Material for M-X-Saber
// Invoker, which detaches them as its cost, and Elfnote Fortuna, a Level 4 monster, to destroy in Defense Position.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (4423206, EXTRA),
    (4423206, EXTRA),
    (23720856, DECK),
    (23720856, DECK),
    (9491461, DECK),
    (9491461, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, gathcon);
    e3.set_target(duel, gathtg);
    e3.set_operation(duel, gathop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e4);
}

fn zbthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ZUBABA]) && c.is_monster(duel) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_defense_pos(duel) && c.is_faceup(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_MZONE, None);
    let b1 = duel.is_existing_matching_card(zbthfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = !g.is_empty();
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_LVCHANGE);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_DESTROY);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, zbthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        {
            duel.confirm_cards(1 - tp, sc)?;
            let lv = sc.get_level(duel);
            let c = e.get_handler(duel);
            if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && !c.is_level(duel, &[lv]) {
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, EFFECT_CHANGE_LEVEL);
                e1.set_value(duel, lv as i32);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
                c.register_effect(duel, e1);
            }
        }
    } else if op == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, desfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn gathcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_COST)
        && a.re.is_some_and(|re| re.is_activated(duel) && re.is_active_type(duel, TYPE_XYZ))
        && c.is_previous_location(duel, LOCATION_OVERLAY)
}

fn gathfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GAGAGA]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn gathtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, gathfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn gathop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, gathfilter, LOCATION_DECK)
}
