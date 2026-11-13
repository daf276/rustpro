// Neo Super Quantal Mech King Blaster Magna
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 95493471;

// On the field, so a copy may sit in an Extra Monster Zone, and in the Extra Deck, Link Summoned from Super Quantum Blue Layers, or the Layers Xyz Summoned into Grampulse,
// with Layers in the Deck to summon and the opponent's Dark Hole to destroy the Xyz Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (85252081, EXTRA),
    (85252081, EXTRA),
    (12369277, MZONE),
    (12369277, MZONE),
    (12369277, MZONE),
    (12369277, MZONE),
    (12369277, DECK),
    (12369277, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_SUPER_QUANT], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_condition(duel, drcon);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn drfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_faceup(duel) && c.is_code(duel, code)
}

fn drcfilter(c: CardId, duel: &Duel, tp: u8, lg: CardSet) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANT])
        && c.is_type(duel, TYPE_XYZ)
        && c.is_previous_location(duel, LOCATION_EXTRA)
        && lg.is_contains(c)
        && !duel.is_existing_matching_card(
            |dc, duel| drfilter(dc, duel, c.get_code(duel)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            Some(c.into()),
        )
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| drcfilter(c, duel, tp, lg), 1, None)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8, zone: u32, rp: u8) -> bool {
    let mut seq = c.get_previous_sequence(duel);
    if !c.is_previous_controler(duel, tp) {
        seq += 16;
    }
    (c.is_reason(duel, REASON_BATTLE) || (c.is_reason(duel, REASON_EFFECT) && rp != tp))
        && c.is_type(duel, TYPE_XYZ)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && (zone >> seq) & 1 != 0
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (tp, rp) = (a.tp, a.rp);
    let zone = a.e.get_handler(duel).get_linked_zone(duel);
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| cfilter(c, duel, tp, zone, rp), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, attr: u32) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANT])
        && c.get_original_attribute(duel) & attr != 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    let zone = e.get_handler(duel).get_linked_zone(duel);
    let g =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| cfilter(c, duel, tp, zone, rp), None);
    let attr = g.get_bitwise_or(duel, |c, duel| u64::from(c.get_original_attribute(duel))) as u32;
    if chk == 0 {
        return Ok(attr != 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, attr), tp, LOCATION_DECK, 0, 1, None));
    }
    e.set_label(duel, attr.into());
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let attr = e.get_label(duel) as u32;
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, attr), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
