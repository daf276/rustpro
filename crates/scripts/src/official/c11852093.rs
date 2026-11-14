// D/D/D Alfred the Divine Sage King
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 11852093;

pub const LISTED_SERIES: &[u16] = &[SET_DD, SET_DDD, SET_DARK_CONTRACT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_DD];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DD], Some(fc), sumtype, tp)); 2];
}

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_DDD])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_able_to_deck(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, |c, duel| c.is_faceup(duel) && c.is_able_to_deck(duel));
        (Some(duel.get_matching_group(filter, tp, LOCATION_REMOVED, 0, None)), None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let location = LOCATION_HAND | LOCATION_MZONE | LOCATION_REMOVED;
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, a.tp, location as i32);
        Ok(false)
    }
}

// On the field with D/D monsters to fuse into D/D/D Flame King Genghis, and in the Graveyard with D/D Necro Slime to be
// banished as its material, with Dark Contract with the Witch there to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (72291412, GRAVE),
    (9765723, GRAVE),
    (72181263, MZONE),
    (33334269, HAND),
    (67322708, HAND),
    (74583607, EXTRA),
    (74583607, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARK_CONTRACT])
        && c.is_continuous_spell_trap(duel)
        && c.is_faceup(duel)
        && c.check_unique_on_field(duel, tp)
        && !c.is_forbidden(duel)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && plfilter(chkc, duel, tp));
    }
    let ddd = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DDD]));
    let ct = duel.get_matching_group_count(ddd, tp, LOCATION_MZONE, 0, None) as i32;
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    let filter = |c, duel: &Duel| plfilter(c, duel, tp);
    if chk == 0 {
        return Ok(ct > 0 && ft > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let max = u8::try_from(ct.min(ft)).unwrap_or(0);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, max, None)?;
    if g.is_some_and(|g| g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None)) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(mut tg) = duel.get_target_cards(a.e).filter(|tg| !tg.is_empty()) else {
        return Ok(());
    };
    let mut gg = CardSet::default();
    let ft = duel.get_location_count(tp, LOCATION_SZONE);
    if ft < tg.len() as i32 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let ft = u8::try_from(ft).unwrap_or(0);
        let sg = tg.filter_select(duel, tp, |c, duel| plfilter(c, duel, tp), ft, ft, None)?;
        gg = tg.difference(sg);
        tg = sg;
    }
    for tc in tg {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    if !gg.is_empty() {
        duel.send_to_grave_full(gg, REASON_RULE | REASON_RETURN, PLAYER_NONE, tp)?;
    }
    Ok(())
}
