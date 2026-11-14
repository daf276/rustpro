// Red Rose Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 26118970;

pub const LISTED_SERIES: &[u16] = &[SET_ROSE_DRAGON];
pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_ROSE_DRAGON];
// Used as Synchro Material with Mystical Elf for Black Rose Dragon in the Extra Deck, with White Rose Dragon in the hand and
// Frozen Rose and Blooming of the Darkest Rose in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (15025844, MZONE),
    (CARD_BLACK_ROSE_DRAGON, EXTRA),
    (CARD_BLACK_ROSE_DRAGON, EXTRA),
    (12213463, HAND),
    (12213463, DECK),
    (53503015, DECK),
    (99092624, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ROSE_DRAGON]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    let rc = e.get_handler(duel).get_reason_card(duel);
    if rc.is_some_and(|rc| rc.is_code(duel, CARD_BLACK_ROSE_DRAGON) || (rc.is_race(duel, RACE_PLANT) && rc.is_type(duel, TYPE_SYNCHRO))) {
        e.set_label(duel, 1);
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    } else {
        e.set_label(duel, 0);
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    }
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, 53503015) || c.is_code(duel, 99092624)) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && e.get_label(duel) == 1
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g2 = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g2.is_empty() {
            duel.send_to_hand(g2, tp, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g2)?;
        }
    }
    Ok(())
}
