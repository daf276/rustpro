// Tellarknight Constellar Delteros
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 32289031;

pub const LISTED_SERIES: &[u16] = &[SET_TELLARKNIGHT, SET_CONSTELLAR];
// Xyz Summoned from the Extra Deck with three Level 4 monsters (Luster Dragon), with Satellarknight Deneb in the Deck to search and
// Alexandrite Dragon (LIGHT) in hand to Normal Summon. The Special Summon needs the card sent to the Graveyard from off the field, and
// nothing ported sends it from the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (11091375, MZONE),
    (11091375, MZONE),
    (11091375, MZONE),
    (75878039, DECK),
    (75878039, DECK),
    (43096270, HAND),
    (43096270, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR]) && c.is_rank_above(duel, 5) && c.is_faceup(duel)
    });
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    Ok(false)
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_summonable(duel, true, None)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        if duel.is_existing_matching_card(nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
            let sg = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
            if let Some(sc) = sg.get_first() {
                duel.break_effect()?;
                duel.summon(tp, sc, true, None)?;
            }
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TELLARKNIGHT, SET_CONSTELLAR])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_REMOVED,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(
        duel,
        a,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_REMOVED,
        POS_FACEUP,
    )
}
