// Thunder Dragon Colossus
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_YESNO;

const ID: u32 = 15291624;

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_THUNDER, Some(fc), sumtype, tp));
    const MINC: i32 = 1;
    const MAXC: i32 = 1;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(31786629)];
}

// In the Extra Deck, with Thunder Dragons in the hand to activate from there and on the field to release or Fuse, and
// one in the Graveyard to banish, and the opponent's Raigeki and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (31786629, HAND),
    (31786629, HAND),
    (31786629, MZONE),
    (31786629, MZONE),
    (31786629, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_SELECT_YESNO)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_range(duel, LOCATION_EXTRA);
    e2.set_condition(duel, hspcon);
    e2.set_target(duel, hsptg);
    e2.set_operation(duel, hspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, LOCATION_DECK);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target(duel, desreptg);
    c.register_effect(duel, e4);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(chainfilter));
}

fn chainfilter(duel: &Duel, re: EffectId, _: u8, cid: u8) -> bool {
    let loc: u32 = duel.get_chain_info(cid, CHAININFO_TRIGGERING_LOCATION);
    !(re.is_monster_effect(duel) && re.get_handler(duel).is_race(duel, RACE_THUNDER) && loc == LOCATION_HAND)
}

fn hspfilter(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    c.is_race(duel, RACE_THUNDER)
        && !c.is_type_full(duel, TYPE_FUSION, Some(sc), MATERIAL_FUSION, tp)
        && c.is_type_full(duel, TYPE_EFFECT, Some(sc), MATERIAL_FUSION, tp)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), sc) > 0
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, |rc, duel| hspfilter(rc, duel, tp, c), 1, release_options(c, tp))
        && (duel.get_custom_activity_count(ID, tp, ACTIVITY_CHAIN) != 0 || duel.get_custom_activity_count(ID, 1 - tp, ACTIVITY_CHAIN) != 0)
}

fn hsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(tp, |rc, duel| hspfilter(rc, duel, tp, c), (1, 1), true, release_options(c, tp))?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    duel.release(g, REASON_COST | REASON_MATERIAL)?;
    c.set_material(duel, Some(g));
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_THUNDER) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && duel.is_existing_matching_card(repfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    if !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(tp, repfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    Ok(true)
}
