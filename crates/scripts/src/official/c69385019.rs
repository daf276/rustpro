// Mermail King - Neptabyss
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 69385019;

pub const LISTED_SERIES: &[u16] = &[SET_ABYSS, SET_ATLANTEAN, SET_MERMAIL];
// On the field against the opponent's Raigeki, with Mermail Abysspike in the Deck to search. No "Abyss-" Equip Spell is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (58471134, DECK),
    (58471134, DECK),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_race_full(duel, RACE_FISH | RACE_SEASERPENT | RACE_AQUA, Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| {
        e.get_handler(duel).get_linked_group(duel).is_contains(c) && c.is_attribute(duel, ATTRIBUTE_WATER)
    });
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, eqcon);
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
    });
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn eqconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_reason(duel, REASON_COST)
        && c.is_attribute(duel, ATTRIBUTE_WATER)
        && (!c.is_previous_location(duel, LOCATION_MZONE) || c.is_previous_attribute_on_field(duel, ATTRIBUTE_WATER))
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_activated(duel)) && a.eg.unwrap_or_default().is_exists(duel, eqconfilter, 1, None)
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8, stzone_check: bool, ec: Option<CardId>) -> bool {
    c.is_set_card(duel, &[SET_ABYSS])
        && c.is_equip_spell(duel)
        && (c.is_able_to_hand(duel)
            || (stzone_check
                && ec.is_some_and(|ec| c.check_equip_target(duel, ec))
                && c.check_unique_on_field(duel, tp)
                && !c.is_forbidden(duel)))
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let stzone_check = duel.get_location_count(tp, LOCATION_SZONE) > 0;
        let ec = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(
            |c, duel| eqfilter(c, duel, tp, stzone_check, Some(ec)),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    let locations = (LOCATION_DECK | LOCATION_GRAVE) as i32;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, locations);
    duel.set_possible_operation_info(0, CATEGORY_EQUIP, None, 1, tp, locations);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let handler = e.get_handler(duel);
    let c = (handler.is_relate_to_effect(duel, e) && handler.is_faceup(duel)).then_some(handler);
    let stzone_check = duel.get_location_count(tp, LOCATION_SZONE) > 0;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = aux::necro_valley_filter(|sc, duel| eqfilter(sc, duel, tp, stzone_check, c));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            sc,
            tp,
            |sc, duel| {
                stzone_check
                    && c.is_some_and(|c| sc.check_equip_target(duel, c))
                    && sc.check_unique_on_field(duel, tp)
                    && !sc.is_forbidden(duel)
            },
            |duel| c.map_or(Ok(()), |c| duel.equip(tp, sc, c).map(drop)),
            aux::stringid(ID, 3),
        )?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ATLANTEAN, SET_MERMAIL]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
