// Gokipole
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 17535764;

// Sent to the Graveyard by the opponent's Raigeki and Dark Hole, with Level 4 Insect Normal Monsters in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (15367030, DECK),
    (53606874, DECK),
    (41762634, DECK),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[4]) && c.is_race(duel, RACE_INSECT) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, tc)?;
    if tc.is_type(duel, TYPE_NORMAL)
        && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        if duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
            return Ok(());
        }
        let attack = tc.get_attack(duel);
        let filter = aux::faceup_filter(move |c, duel| c.is_attack_above(duel, attack));
        let dg = duel.get_matching_group(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
        if !dg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let dc = dg.select(duel, tp, 1, 1, None)?;
            if !dc.is_empty() {
                duel.break_effect()?;
                duel.hint_selection_full(dc, true);
                duel.destroy(dc, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
