// Aiwass, the Magistus Spell Spirit
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 35877582;

pub const LISTED_SERIES: &[u16] = &[SET_MAGISTUS];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_MAGISTUS], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Crowley and Mystical Elf, and on the field, with the opponent's
// monster to equip to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (875572, HAND),
    (15025844, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}

fn eqfilter(c: CardId, duel: &Duel, tp: u8, ft: i32, mmz: bool) -> bool {
    c.is_faceup(duel) && (ft > 0 || c.is_controler(duel, tp) || mmz)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let mmz = c.get_sequence(duel) < 5;
    // `not chkc==c` parses as `(not chkc)==c`, which is never true.
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(
                |c, duel| eqfilter(c, duel, tp, ft, mmz),
                tp,
                LOCATION_MZONE,
                LOCATION_MZONE,
                1,
                Some(c.into()),
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(
        tp,
        |c, duel| eqfilter(c, duel, tp, ft, mmz),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if !(c.is_relate_to_effect(duel, e) && c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel)) {
        return Ok(());
    }
    let tc = duel.get_first_target();
    let full = duel.get_location_count(tp, LOCATION_SZONE) <= 0;
    unported!(duel, tc.is_none() && !full);
    let tc = tc.filter(|tc| !full && !tc.is_facedown(duel) && tc.is_relate_to_effect(duel, e));
    let Some(tc) = tc else {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(());
    };
    duel.equip_full(tp, c, tc, true, false)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
    e1.set_label_object_group(duel, Some(tc.into()));
    c.register_effect(duel, e1);
    if tc.is_controler(duel, 1 - tp) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_code(duel, EFFECT_CANNOT_TRIGGER);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_EQUIP);
        e3.set_code(duel, EFFECT_SET_CONTROL);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e3.set_value(duel, Value::Card(ctval));
        c.register_effect(duel, e3);
    }
    Ok(())
}

fn ctval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    i32::from(e.get_handler_player(duel))
}
