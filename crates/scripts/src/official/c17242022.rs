// Red-Eyes Black Dragon Exceed
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 17242022;

pub const LISTED_NAMES: &[u32] = &[CARD_REDEYES_B_DRAGON, CARD_DARK_TIME_WIZARD, ID];
pub const MATERIAL_SETCODE: &[u16] = &[SET_RED_EYES];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_REDEYES_B_DRAGON),
        FusionMaterial::Filter(|c, duel, _, _, _| c.lists_code(duel, CARD_DARK_TIME_WIZARD)),
    ];
}

// In the Extra Deck, Fusion Summoned with Red-Eyes Black Dragon and Dragon Knight, or after Dark Time Wizard destroys Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_REDEYES_B_DRAGON, HAND),
    (4881365, HAND),
    (40235813, HAND),
    (40235813, HAND),
    (40235813, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (1784619, MZONE),
    (1784619, HAND),
    (1784619, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e0a = duel.create_effect(c);
    e0a.set_description(duel, aux::stringid(ID, 0));
    e0a.set_type(duel, EFFECT_TYPE_FIELD);
    e0a.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0a.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0a.set_range(duel, LOCATION_EXTRA);
    e0a.set_condition(duel, selfspcon);
    e0a.set_target(duel, selfsptg);
    e0a.set_operation(duel, selfspop);
    e0a.set_value(duel, 1);
    c.register_effect(duel, e0a);
    let e0b = duel.create_effect(c);
    e0b.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e0b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0b.set_condition(duel, regcon);
    e0b.set_operation(duel, regop);
    c.register_effect(duel, e0b);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD | EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DESTROYED);
        ge1.set_operation(duel, |duel, a| {
            unported!(duel, a.r & REASON_EFFECT > 0 && a.re.is_none());
            if a.r & REASON_EFFECT > 0 && a.re.is_some_and(|re| re.get_handler(duel).is_code(duel, CARD_DARK_TIME_WIZARD)) {
                duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
            }
            Ok(())
        });
        duel.register_effect(ge1, 0);
    });
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, lv8sptg);
    e1.set_operation(duel, lv8spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Effect(|duel, e, te| {
            te.get_owner_player(duel) == 1 - e.get_handler_player(duel)
                && te.is_activated(duel)
                && (te.is_monster_effect(duel) || te.is_spell_effect(duel))
        }),
    );
    c.register_effect(duel, e2);
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8, fc: CardId) -> bool {
    c.is_releasable(duel)
        && c.is_faceup(duel)
        && c.is_can_be_fusion_material_full(duel, Some(fc), MATERIAL_FUSION, tp)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), fc) > 0
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        use_oppo: true,
        ..ReleaseOptions::new(tp)
    }
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_flag_effect(0, ID) > 0
        && duel.check_release_group_full(tp, |rc, duel| selfspcostfilter(rc, duel, tp, c), 1, release_options(c, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(
        tp,
        |rc, duel| selfspcostfilter(rc, duel, tp, c),
        (1, 1),
        true,
        release_options(c, tp),
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    if !g.is_empty() {
        duel.release(g, REASON_COST | REASON_MATERIAL)?;
    }
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_fusion_summoned(duel) || c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        c.is_original_code_rule(duel, ID)
            && (sumtype & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION || sumtype & (SUMMON_TYPE_SPECIAL + 1) == SUMMON_TYPE_SPECIAL + 1)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn lv8spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 8) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn lv8sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| lv8spfilter(c, duel, a.e, a.tp),
        LOCATION_HAND | LOCATION_GRAVE,
    )
}

fn lv8spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| lv8spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
