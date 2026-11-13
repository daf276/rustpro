// Subterror Behemoth Fiendess
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 74937659;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_FLIP, Some(lc), sumtype, tp)
    }
}

// Flip monsters (Hane-Hane, Armed Ninja) to Link Summon with, to send and to Set, and an opponent's attacker (Elfnote Lucina)
// to flip them. No Subterror monster is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (7089711, MZONE),
    (7089711, MZONE),
    (9076207, MZONE),
    (7089711, DECK),
    (7089711, DECK),
    (9076207, HAND),
    (9076207, HAND),
    (13597785, MZONE | OPPONENT),
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
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, ID);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_FLIP);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, thcon);
    e3.set_target(
        duel,
        stock::operation_info_target::<CATEGORY_TOHAND, { LOCATION_DECK | LOCATION_GRAVE }>,
    );
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c
        .get_material(duel)
        .filter(duel, |mc, duel| mc.is_set_card(duel, &[SET_SUBTERROR]), None);
    let atk = g.get_sum(duel, |mc, duel| i64::from(mc.get_original_level(duel)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, atk as i32 * 100);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FLIP) && c.is_able_to_grave(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, zone)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(g, REASON_EFFECT)? != 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, zone), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if zone != 0 && !sg.is_empty() {
            duel.special_summon_full(sg, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE, zone)?;
        }
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, _| lg.is_contains(c), 1, None))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FLIP) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_DECK | LOCATION_GRAVE)
}
