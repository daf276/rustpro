// Codebreaker Virus Swordsman
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 84121193;

pub const LISTED_SERIES: &[u16] = &[SET_CODEBREAKER];

const ZERO_DAY: u32 = 8662794;

pub const LISTED_NAMES: &[u32] = &[ZERO_DAY];

// Codebreaker Virus Berserkers on the field, whose vertical arrows make a mutual link with it when it is Link Summoned
// from Zero Days or revived by Monster Reborn, Zero Days to Special Summon, and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (ID, GRAVE),
    (ZERO_DAY, MZONE),
    (ZERO_DAY, MZONE),
    (ZERO_DAY, DECK),
    (ZERO_DAY, HAND),
    (48736598, MZONE),
    (48736598, MZONE),
    (CARD_MONSTER_REBORN, HAND),
    (CARD_MONSTER_REBORN, HAND),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_mutual_linked_group_count(duel) > 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e3.set_target(duel, stock::special_summon_self_target_player_0);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, p: u8, zones: u32) -> bool {
    c.is_code(duel, ZERO_DAY) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, p, zones)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok([tp, 1 - tp].into_iter().any(|p| {
            let zones = aux::get_mm_zones_pointed_to(duel, p);
            duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, p, zones), tp, locations, 0, 1, None)
        }));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let zones = [aux::get_mm_zones_pointed_to(duel, 0), aux::get_mm_zones_pointed_to(duel, 1)];
    let zone = |p: u8| zones.get(usize::from(p)).copied().unwrap_or(0);
    let group = |duel: &Duel, p: u8| {
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, p, zone(p)));
        duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, None)
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = group(duel, tp).union(group(duel, 1 - tp));
    let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    let own = spfilter(tc, duel, e, tp, tp, zone(tp));
    let p = if own && spfilter(tc, duel, e, tp, 1 - tp, zone(1 - tp)) {
        if duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            1 - tp
        } else {
            tp
        }
    } else if own {
        tp
    } else {
        1 - tp
    };
    duel.special_summon_full(tc, 0, tp, p, false, false, POS_FACEUP, zone(p))?;
    Ok(())
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_reason(duel, REASON_DESTROY) && a.rp != a.tp {
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
