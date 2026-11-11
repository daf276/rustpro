// Pumpking the King of Grave Ghosts
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, SZONE};

const ID: u32 = 81684048;

pub const LISTED_NAMES: &[u32] = &[CARD_CALL_OF_THE_HAUNTED];
// Two copies in the hand (one to reveal, one to discard) and Call of the Haunted in the Deck to Set, and one in the Graveyard with
// Call of the Haunted set to Special Summon it, with Great Mammoth of the Netherworld, a Level 6 Zombie, in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (CARD_CALL_OF_THE_HAUNTED, DECK),
    (CARD_CALL_OF_THE_HAUNTED, DECK),
    (CARD_CALL_OF_THE_HAUNTED, SZONE),
    (80461466, DECK),
    (80461466, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET + CATEGORY_TOGRAVE + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_CALL_OF_THE_HAUNTED) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None)
                && duel.is_existing_matching_card(
                    |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
                    tp,
                    LOCATION_HAND,
                    0,
                    1,
                    None,
                ),
        );
    }
    duel.set_operation_info(0, CATEGORY_SET, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(setfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first()
        && duel.sset(tp, sc)? > 0
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        sc.register_effect(duel, e1);
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[6])
        && c.is_race(duel, RACE_ZOMBIE)
        && !c.is_attack(duel, &[1950])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_HAND | LOCATION_GRAVE) && c.is_race_except(duel, RACE_ZOMBIE)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
