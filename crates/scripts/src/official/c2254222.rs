// Speedroid Wing Synchron
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, PZONE};

const ID: u32 = 2254222;

pub const LISTED_NAMES: &[u32] = &[82044279];

// Speedroid Dominobutterfly (Level 2 WIND) in the other Pendulum Zone, Clear Wing Synchro Dragon in the Extra Deck and Adamancipator Risen - Raptite
// (a WIND Synchro of another race) on the field, Speedroid Scratch (a Speedroid Spell) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (28151978, PZONE),
    (82044279, EXTRA),
    (73079836, MZONE),
    (29114773, DECK),
    (29114773, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_DECK);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, tgcon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[2]) && c.is_attribute(duel, ATTRIBUTE_WIND)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 82044279)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(desfilter, tp, LOCATION_PZONE, 0, 1, Some(e.get_handler(duel).into()))
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None),
        );
    }
    let g = duel.get_field_group(tp, LOCATION_PZONE, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let pc = duel
        .get_matching_group(aux::TRUE, tp, LOCATION_PZONE, 0, Some(c.into()))
        .get_first();
    if c.is_relate_to_effect(duel, e)
        && let Some(pc) = pc
        && desfilter(pc, duel)
        && duel.destroy(CardSet::from_iter([c, pc]), REASON_EFFECT)? == 2
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sc = duel
            .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
            .get_first();
        if let Some(sc) = sc
            && duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0
        {
            sc.complete_procedure(duel);
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_target_range(duel, LOCATION_PZONE, 0);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 3));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_card(duel, |duel, _, c| !c.is_attribute(duel, ATTRIBUTE_WIND));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel)
}

fn windsynchrofilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WIND) && c.is_type(duel, TYPE_SYNCHRO) && c.is_faceup(duel)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SPEEDROID]) && c.is_spell_trap(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(windsynchrofilter, tp, LOCATION_MZONE, 0, None);
        return Ok(
            duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None) && g.get_bin_class_count(duel, CardId::get_race) >= 2
        );
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
