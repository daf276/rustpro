// Arionpos, Serpent of the Ghoti
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 65910922;

// A copy on the field and three in the Extra Deck, Synchro Summoned with Paces (a Level 2 Tuner) and Great White (Level 4), and as
// a material of Askaan with Paces, with more Fish monsters in the Deck to banish and add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (46815301, EXTRA),
    (46815301, EXTRA),
    (46037983, MZONE),
    (46037983, MZONE),
    (13429800, MZONE),
    (92409659, DECK),
    (13429800, DECK),
    (13429800, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rmvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcond);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn rmvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FISH) && c.is_level_below(duel, 6) && c.is_able_to_remove(duel)
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, rmvfilter, LOCATION_DECK, CATEGORY_REMOVE)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, rmvfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn thcond(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r & REASON_SYNCHRO > 0
}

fn tgtfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FISH) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tgtfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tgtfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, tgtfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_race(duel, RACE_FISH) && c.is_level_below(duel, lv) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_race(duel, RACE_FISH) && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0 {
        let lv = tc.get_level(duel);
        if duel.is_existing_matching_card(|c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
    }
    Ok(())
}
