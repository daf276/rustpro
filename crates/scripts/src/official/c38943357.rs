// Spell Power Mastery
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 38943357;
const SPELL_POWER_GRASP: u32 = 75014062;

pub const LISTED_NAMES: &[u32] = &[SPELL_POWER_GRASP];
// Skilled White Magician and Spell Power Grasp on the field to count and place counters on, Endymion cards in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (46363422, MZONE),
    (SPELL_POWER_GRASP, 0),
    (96228804, DECK),
    (96228804, DECK),
    (7656689, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENDYMION]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn counter_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_can_add_counter(duel, COUNTER_SPELL, 1)),
        tp,
        LOCATION_ONFIELD,
        0,
        None,
    )
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? <= 0 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    let named = aux::faceup_filter(|c, duel| c.is_code(duel, ID) || c.is_code(duel, SPELL_POWER_GRASP));
    let mut ct = duel.get_matching_group_count(named, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, None);
    let mut cg = counter_group(duel, tp);
    if ct > 0 && !cg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.break_effect()?;
        while ct > 0 && !cg.is_empty() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_COUNTER);
            if let Some(tc) = cg.select(duel, tp, 1, 1, None)?.get_first() {
                tc.add_counter(duel, COUNTER_SPELL, 1);
            }
            ct -= 1;
            cg = counter_group(duel, tp);
        }
    }
    Ok(())
}
