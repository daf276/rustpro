// Exceptional Schedule
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 52782439;
const TOKEN_SCHEDULE: u32 = ID + 1;
const SPECIAL_SCHEDULE: u32 = 97520701;
const URGENT_SCHEDULE: u32 = 25274141;

pub const LISTED_NAMES: &[u32] = &[SPECIAL_SCHEDULE, URGENT_SCHEDULE, TOKEN_SCHEDULE];
// In hand and in the Graveyard, with the Schedules in the Deck, Jizukiru (Level 10 Machine) in the Graveyard to Special
// Summon and Overdrive on the field to send there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (SPECIAL_SCHEDULE, DECK),
    (URGENT_SCHEDULE, DECK),
    (63941210, GRAVE),
    (2311603, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_SCHEDULE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOKEN + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, SPECIAL_SCHEDULE) || c.is_code(duel, URGENT_SCHEDULE)) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        let stats = MonsterStats::new(0, TYPES_TOKEN, 3000, 3000, 10, RACE_MACHINE, ATTRIBUTE_EARTH);
        if duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
            && duel.is_player_can_special_summon_monster_full(tp, TOKEN_SCHEDULE, stats, POS_FACEUP, 1 - tp, 0)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
            && let Some(token) = duel.create_token(tp, TOKEN_SCHEDULE)
        {
            duel.break_effect()?;
            duel.special_summon(token, 0, tp, 1 - tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.get_mzone_count(tp, Some(c.into())) > 0 && c.is_able_to_grave(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[10])
        && c.is_race(duel, RACE_MACHINE)
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_GRAVE | LOCATION_REMOVED;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, locations) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, None)
                && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_ONFIELD as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(sc) = duel
        .select_matching_card(tp, |c, duel| tgfilter(c, duel, tp), tp, LOCATION_ONFIELD, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    if duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
