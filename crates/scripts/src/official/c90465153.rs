// Mannadium Prime-Heart
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 90465153;

pub const LISTED_SERIES: &[u16] = &[SET_MANNADIUM];
pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN1: usize = 1;
    const MAX1: usize = 99;
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned (Level 10) with Mannadium Trisukta (a Level 6 Tuner) and Fiend Reflection #2 or Gradius
// (Level 4 LIGHT monsters), or with Guardragon Justicia (Level 2 Tuner) and Hourglass of Life (Level 2 LIGHT) as well. No ported
// monster has 1500 ATK and 2100 DEF, or is Visas Starfrost, for the Special Summon. The hint is written while the card is in the
// Extra Deck, where the stream names no card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (21368273, MZONE),
    (2863439, MZONE),
    (10992251, MZONE),
    (11012154, MZONE),
    (8783685, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Report(&Check::CardHint(aux::stringid(ID, 1))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e0| e0.get_label(duel) >= 2)
    });
    e1.set_operation(duel, atkop);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, tgcond);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn tunerfilter(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
    let sumtype = SUMMON_TYPE_SYNCHRO as u64;
    c.is_set_card_full(duel, &[SET_MANNADIUM], Some(sc), sumtype, tp) && c.is_type_full(duel, TYPE_TUNER, Some(sc), sumtype, tp)
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let mg = c.get_material(duel);
    e.set_label(duel, mg.filter_count(duel, |c, duel| c.is_type(duel, TYPE_TUNER), None) as i64);
    let tp = e.get_handler_player(duel);
    if mg.is_exists(duel, |tc, duel| tunerfilter(tc, duel, c, tp), 1, None) {
        let reset = RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE));
        c.register_flag_effect_full(duel, ID, reset, EFFECT_FLAG_CLIENT_HINT, 1, 0, aux::stringid(ID, 1));
    }
    if mg.is_exists(
        duel,
        |c, duel| c.is_type(duel, TYPE_TUNER) || c.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some(),
        2,
        None,
    ) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
        e1.set_code(duel, EFFECT_MULTIPLE_TUNERS);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD) | RESET_PHASE | PHASE_END);
        c.register_effect(duel, e1);
    }
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let label = a.e.get_label_object(duel).map_or(0, |e0| e0.get_label(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_value(duel, (label - 1) as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}

fn tgcond(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_synchro_summoned(duel) && c.has_flag_effect(duel, ID)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.rp == 1 - tp
        && c.is_previous_controler(duel, tp)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_synchro_summoned(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_VISAS_STARFROST) || (c.is_attack(duel, &[1500]) && c.is_defense(duel, &[2100])))
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_GRAVE | LOCATION_REMOVED,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
