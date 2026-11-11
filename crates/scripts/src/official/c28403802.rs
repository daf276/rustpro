// P.U.N.K. JAM Dragon Drive
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 28403802;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PSYCHIC, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned from the Extra Deck with Ze Amin (Level 3 Psychic Tuner) and Shapesnatch (Level 5), with Level 3 Psychics in the Deck, and a copy in the Graveyard to respond to the opponent's Ash Blossoms chaining to Wa Gon's search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (19535693, MZONE),
    (19535693, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (50642380, DECK),
    (82041999, DECK),
    (ID, GRAVE),
    (50642380, MZONE),
    (82041999, MZONE),
    (17691568, DECK),
    (14558127, HAND | OPPONENT),
    (14558127, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, thgcon);
    e1.set_cost(duel, Cost::pay_lp::<600>);
    e1.set_target(duel, thgtg);
    e1.set_operation(duel, thgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn thgcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel) || a.re.is_some_and(|re| re.get_handler(duel).is_set_card(duel, &[SET_PUNK]))
}

fn thgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[3]) && c.is_race(duel, RACE_PSYCHIC) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thgfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn thgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    if let Some(tc) = duel
        .select_matching_card(tp, thgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            CardId::is_able_to_grave,
            |duel| duel.send_to_grave(tc, REASON_EFFECT).map(drop),
            574,
        )?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let ch = i32::from(duel.get_current_chain()) - 1;
    if ch <= 0 {
        return false;
    }
    let cplayer: u8 = duel.get_chain_info(ch as u8, CHAININFO_TRIGGERING_CONTROLER);
    let ceff: Option<EffectId> = duel.get_chain_info(ch as u8, CHAININFO_TRIGGERING_EFFECT);
    a.ep == 1 - tp && cplayer == tp && ceff.is_some_and(|ceff| ceff.get_handler(duel).is_set_card(duel, &[SET_PUNK]))
}
