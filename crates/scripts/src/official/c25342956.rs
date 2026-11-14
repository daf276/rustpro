// Gem-Knight Master Diamond Dispersion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 25342956;

pub const LISTED_SERIES: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_GEM], Some(fc), sumtype, tp)); 3];
}

// On the field, with Gem-Knight Garnet in the GY and Gem-Knight Ruby and Seraphinite in the Extra Deck to summon; these and
// Gem-Knight Amethyst destroyed by the opponent's Gem-Knight Master Diamond in battle, with a copy in the GY, for its return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (91731841, GRAVE),
    (91731841, GRAVE),
    (76614340, EXTRA),
    (3113836, EXTRA),
    (71616908, MZONE),
    (71616908, MZONE),
    (39512984, MZONE | OPPONENT),
    (39512984, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(76614340),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DESTROYED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, selfspcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, hc: Option<CardId>) -> bool {
    if !(c.is_set_card(duel, &[SET_GEM])
        && !c.is_race(duel, RACE_ROCK)
        && c.is_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, hc.map(CardSet::from), c) > 0
    } else {
        duel.get_mzone_count(tp, hc.map(CardSet::from)) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_GRAVE | LOCATION_EXTRA;
    if chk == 0 {
        let hc = e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, Some(hc)), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn exfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_EXTRA) && c.is_facedown(duel) && c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ)
}

fn exfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_EXTRA) && (c.is_type(duel, TYPE_LINK) || (c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM)))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft1 = duel.get_location_count(tp, LOCATION_MZONE);
    let mut ft2 = duel.get_location_count_from_ex(tp, tp, None, None);
    let mut ft3 = duel.get_location_count_from_ex(tp, tp, None, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ);
    let mut ft4 = duel.get_location_count_from_ex(tp, tp, None, TYPE_PENDULUM | TYPE_LINK);
    let mut ft = duel.get_usable_mzone_count(tp).min(3);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        for count in [&mut ft1, &mut ft2, &mut ft3, &mut ft4] {
            if *count > 0 {
                *count = 1;
            }
        }
        ft = 1;
    }
    if let Some(ect) = aux::check_summon_gate(duel, tp) {
        for count in [&mut ft1, &mut ft2, &mut ft3, &mut ft4] {
            *count = ect.min(*count);
        }
    }
    let mut location = 0;
    if ft1 > 0 {
        location = LOCATION_GRAVE;
    }
    if ft2 > 0 || ft3 > 0 || ft4 > 0 {
        location |= LOCATION_EXTRA;
    }
    if location > 0 {
        let sg = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, None)),
            tp,
            location,
            0,
            None,
        );
        if !sg.is_empty() {
            let rescon = |duel: &Duel, sg: CardSet| {
                let exnpct = sg.filter_count(duel, exfilter1, None) as i32;
                let expct = sg.filter_count(duel, exfilter2, None) as i32;
                let mct = sg.filter_count(duel, |c, duel| !c.is_location(duel, LOCATION_EXTRA), None) as i32;
                let groupcount = sg.len();
                let classcount = sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
                let res = ft3 >= exnpct && ft4 >= expct && ft1 >= mct && ft >= groupcount as i32 && classcount == groupcount;
                (res, !res)
            };
            let rg = aux::select_unselect_group(
                duel,
                sg,
                1,
                ft.max(0) as usize,
                &|duel, sg, _, _| rescon(duel, sg),
                tp,
                HINTMSG_SPSUMMON,
            )?;
            duel.special_summon(rg, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_FUSION)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_FUSION));
    Ok(())
}

fn selfspconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_previous_type_on_field(duel, TYPE_FUSION) && c.is_previous_controler(duel, tp)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| selfspconfilter(c, duel, tp), 1, None))
}
