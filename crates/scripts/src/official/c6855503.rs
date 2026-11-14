// Masquerade the Blazing Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 6855503;

pub const LISTED_SERIES: &[u16] = &[SET_DESPIA];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DESPIA], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(matfilter),
    ];
}

// In the GY against the opponent's Despian Quaeritis, and in the Extra Deck with Polymerization and two Aluber the
// Jester of Despia in hand for its Fusion Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, EXTRA),
    (72272462, MZONE | OPPONENT),
    (24094653, HAND),
    (62962630, HAND),
    (62962630, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_ACTIVATE_COST);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, costcon);
    e1.set_cost_activate(duel, |duel, _, _, tp| {
        duel.check_lp_cost(tp, duel.get_player_effect(tp, ID).len() as u32 * 600)
    });
    e1.set_operation(duel, |duel, a| duel.pay_lp_cost(a.tp, 600));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, ID);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, costcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(spcfilter, a.tp, 0, LOCATION_MZONE, 1, None)
    });
    e3.set_target(duel, stock::special_summon_self_target_player_0);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, st: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(fc), st, tp) || c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), st, tp)
}

fn costcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_fusion_summoned(duel)
}

fn spcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_RITUAL + TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ + TYPE_LINK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
