// Small World
use crate::prelude::*;

const ID: u32 = 89558743;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn singleprop(c: CardId, duel: &Duel, d: CardId) -> bool {
    let matches = [
        c.is_race(duel, d.get_race(duel)),
        c.is_attribute(duel, d.get_attribute(duel)),
        c.is_level(duel, &[d.get_level(duel)]),
        c.is_attack(duel, &[d.get_text_attack(duel)]) && d.is_attack(duel, &[c.get_text_attack(duel)]),
        c.is_defense(duel, &[d.get_text_defense(duel)]) && d.is_defense(duel, &[c.get_text_defense(duel)]),
    ];
    matches.into_iter().filter(|&m| m).count() == 1
}

fn thfilter(c: CardId, duel: &Duel, mc: CardId) -> bool {
    c.is_monster(duel) && c.is_able_to_hand(duel) && singleprop(c, duel, mc)
}

fn showfilter(c: CardId, duel: &Duel, tp: u8, mc: CardId) -> bool {
    c.is_monster(duel)
        && !c.is_public(duel)
        && c.is_able_to_remove(duel)
        && singleprop(c, duel, mc)
        && duel.is_existing_matching_card(|t, duel| thfilter(t, duel, c), tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn revfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && !c.is_public(duel)
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|s, duel| showfilter(s, duel, tp, c), tp, LOCATION_DECK, 0, 1, None)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| revfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 2, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let Some(rc) = duel
        .select_matching_card(tp, |c, duel| revfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let Some(sc) = duel
        .select_matching_card(tp, |c, duel| showfilter(c, duel, tp, rc), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.confirm_cards(1 - tp, rc)?;
    duel.confirm_cards(1 - tp, sc)?;
    if duel.remove(rc, POS_FACEDOWN, REASON_EFFECT)? > 0 && rc.is_location(duel, LOCATION_REMOVED) && rc.is_facedown(duel) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let th = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, sc), tp, LOCATION_DECK, 0, 1, 1, Some(sc.into()))?;
        if !th.is_empty() {
            duel.break_effect()?;
            if duel.send_to_hand(th, PLAYER_NONE, REASON_EFFECT)? > 0 && th.get_first().is_some_and(|c| c.is_location(duel, LOCATION_HAND))
            {
                duel.confirm_cards(1 - tp, th)?;
                duel.remove(sc, POS_FACEDOWN, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
