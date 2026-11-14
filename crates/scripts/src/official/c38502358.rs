// Mekk-Knight Spectrum Supreme
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 38502358;

pub const LISTED_SERIES: &[u16] = &[SET_MEKK_KNIGHT];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_MEKK_KNIGHT], Some(lc), sumtype, tp)
    }
}

// Link Summoned from Mekk-Knight Avram (and Girsu) in the Extra Monster Zone, with more Mekk-Knights in the Deck and on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (84754430, MZONE),
    (84754430, MZONE),
    (69811710, MZONE),
    (84754430, MZONE),
    (84754430, DECK),
    (69811710, DECK),
    (84754430, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_column_group_count(duel) == 0);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.get_sequence(duel) > 4 && c.get_linked_group_count(duel) == 0
    });
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_cost(duel, spcost);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn spcfilter(c: CardId, duel: &Duel, hc: CardId) -> bool {
    c.get_column_group(duel).is_contains(hc) && c.is_able_to_grave_as_cost(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MEKK_KNIGHT]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let loc = if ft <= 0 { LOCATION_MZONE } else { LOCATION_ONFIELD };
    if chk == 0 {
        return Ok(ft > -1 && duel.is_existing_matching_card(|sc, duel| spcfilter(sc, duel, c), tp, loc, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |sc, duel| spcfilter(sc, duel, c), tp, loc, 0, 1, 1, Some(c.into()))?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP_DEFENSE)
}
