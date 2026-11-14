// Evil Lord - Zorc
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 73320394;

pub const ROLL_DICE: bool = true;
// In the hand, the Graveyard and on the field, with Level 8 Fiends in the Deck to search, Kozaky on the field to be destroyed, and
// Elfnote Lucina on the opponent's field to destroy it by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, MZONE),
    (1966438, DECK),
    (1966438, DECK),
    (99171160, MZONE),
    (99171160, MZONE),
    (99171160, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DICE + CATEGORY_CONTROL + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, dicetg);
    e3.set_operation(duel, diceop);
    c.register_effect(duel, e3);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(
            c.is_discardable(duel) && duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into()))
        );
    }
    let mut sg = CardSet::from(c);
    let mut g = duel.get_matching_group(CardId::is_discardable, tp, LOCATION_HAND, 0, Some(c.into()));
    while sg.len() < 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
        let sc = g.select_unselect(duel, Some(sg), tp, false, false, 2, 2)?;
        unported!(duel, sc.is_none());
        let Some(sc) = sc else { break };
        if !sg.is_contains(sc) {
            sg.insert(sc);
            g.remove(sc);
        } else if sc != c {
            sg.remove(sc);
            g.insert(sc);
        }
    }
    duel.send_to_grave(sg, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[8]) && c.is_race(duel, RACE_FIEND) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_controler(duel, tp) && c.is_previous_race_on_field(duel, RACE_FIEND)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| spconfilter(c, duel, tp), 1, None))
}

fn dicetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_CONTROL, None, 1, 1 - tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn diceop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(&dice) = duel.toss_dice(tp, 1)?.first() else {
        return Ok(());
    };
    if (1..=4).contains(&dice) {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
        let Some(sc) = duel
            .select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.hint_selection(sc);
        let b1 = sc.is_able_to_change_controler(duel);
        match duel.select_effect(tp, &[(b1, aux::stringid(ID, 4)), (true, aux::stringid(ID, 5))])? {
            Some(1) => {
                duel.get_control(sc, tp)?;
            }
            Some(2) => {
                duel.destroy(sc, REASON_EFFECT)?;
            }
            _ => {}
        }
    } else if dice == 5 || dice == 6 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
