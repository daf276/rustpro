// Celestial Knightlord Parshath
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 48589580;
pub const LISTED_NAMES: &[u32] = &[CARD_SANCTUARY_SKY];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_FAIRY, Some(lc), sumtype, tp)
    }
}

// Fairies (Key Mace, Mokey Mokey) to Link Summon with and to send, Artifact Lancea in the hand to summon, The Sanctuary
// in the Sky to search, and the opponent's Dark Hole to destroy them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (1929294, MZONE),
    (1929294, MZONE),
    (27288416, MZONE),
    (1929294, GRAVE),
    (34267821, HAND),
    (34267821, HAND),
    (56433456, DECK),
    (56433456, DECK),
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
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, LOCATION_HAND>);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel, sanc_chk: bool) -> bool {
    c.is_able_to_hand(duel)
        && (c.is_code(duel, CARD_SANCTUARY_SKY) || c.lists_code(duel, CARD_SANCTUARY_SKY) || (sanc_chk && c.is_race(duel, RACE_FAIRY)))
}

fn sanc_chk(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_code(duel, CARD_SANCTUARY_SKY),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        None,
    ) || duel.is_environment(CARD_SANCTUARY_SKY)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let sanc_chk = sanc_chk(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, sanc_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let sanc_chk = sanc_chk(duel, tp);
    stock::search(duel, a, |c, duel| thfilter(c, duel, sanc_chk), LOCATION_DECK)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_previous_race_on_field(duel) & RACE_FAIRY == RACE_FAIRY
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, tp), 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_race(duel, RACE_FAIRY) && c.get_level(duel) > lv && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FAIRY)
        && c.has_level(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_level(duel)),
            tp,
            LOCATION_HAND,
            0,
            1,
            None,
        )
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spcfilter(c, duel, e, tp), tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| spcfilter(c, duel, e, tp),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    if let Some(tc) = g.get_first() {
        let level = tc.get_level(duel);
        e.set_label(duel, i64::from(level));
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let lv = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, lv), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
