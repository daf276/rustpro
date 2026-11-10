// Spirit with Eyes of Blue
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 42097666;
const MAUSOLEUM_OF_WHITE: u32 = 24382602;

pub const LISTED_NAMES: &[u32] = &[MAUSOLEUM_OF_WHITE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_level_below(duel, 4) && c.is_race_full(duel, RACE_DRAGON | RACE_SPELLCASTER, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned from Level 4 Dragons, with Mausoleum of White in the Deck and Blue-Eyes monsters (one
// with an effect) in the hand and the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (11091375, MZONE),
    (43096270, MZONE),
    (MAUSOLEUM_OF_WHITE, DECK),
    (MAUSOLEUM_OF_WHITE, DECK),
    (MAUSOLEUM_OF_WHITE, DECK),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (38517737, GRAVE),
    (38517737, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_DRAGON));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e2.set_target(duel, thtgtg);
    e2.set_operation(duel, thtgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_tribute);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, MAUSOLEUM_OF_WHITE) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thtgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, thtgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            CardId::is_able_to_grave,
            |duel| duel.send_to_grave(sc, REASON_EFFECT).map(drop),
            574,
        )?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_BLUE_EYES]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let handler = e.get_handler(duel);
        return Ok(duel.get_mzone_count(tp, Some(handler.into())) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
        && sc.is_type(duel, TYPE_EFFECT)
        && sc.is_summon_location(duel, LOCATION_GRAVE)
    {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3206);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
        sc.negate_effects(duel, c, 0, false);
    }
    duel.special_summon_complete()?;
    Ok(())
}
