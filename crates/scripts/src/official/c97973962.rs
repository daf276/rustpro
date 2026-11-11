// Aleister the Invoker of Madness
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 97973962;
const INVOCATION: u32 = 74063034;
const BOOK_OF_THE_LAW: u32 = 458748;
const OMEGA_SUMMON: u32 = 47457347;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| c.get_race(duel))
            && aux::check_different_property_binary(duel, sg, &|c, duel| u64::from(c.get_attribute(duel)))
    }
}

pub const LISTED_NAMES: &[u32] = &[INVOCATION, BOOK_OF_THE_LAW, OMEGA_SUMMON];

// On the field with The Book of the Law in the hand to Fusion Summon an Invoked monster (Raidjin to Tribute, Sorath in the Extra
// Deck), Invocation, the Book and Omega Summon in the Deck to search, and Dark Hole in the opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (BOOK_OF_THE_LAW, HAND),
    (49513164, MZONE),
    (43989315, EXTRA),
    (INVOCATION, DECK),
    (BOOK_OF_THE_LAW, DECK),
    (OMEGA_SUMMON, DECK),
    (OMEGA_SUMMON, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, 86120751);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_condition(duel, thcon2);
    e3.set_target(duel, thtg2);
    e3.set_operation(duel, thop2);
    c.register_effect(duel, e3);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, CardId::is_fusion_summoned, 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, INVOCATION) || c.is_code(duel, BOOK_OF_THE_LAW)) && c.is_able_to_hand(duel)
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None)
            && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)? != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn thcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_reason_player(duel) != a.tp
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_previous_controler(duel) == a.tp
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, OMEGA_SUMMON) && c.is_able_to_hand(duel)
}

fn thtg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter2, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop2(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter2, LOCATION_DECK)
}
