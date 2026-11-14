// D/D/D/D Dimensional King Arc Crisis
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, PZONE, SZONE};

const ID: u32 = 71398055;
const SPECIAL_SUMMON_PROCEDURE: u32 = SUMMON_TYPE_SPECIAL + 1;

pub const LISTED_SERIES: &[u16] = &[SET_DARK_CONTRACT, SET_DOOM_KING];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| matfilter(c, duel, fc, sumtype, tp, TYPE_FUSION)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| matfilter(c, duel, fc, sumtype, tp, TYPE_SYNCHRO)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| matfilter(c, duel, fc, sumtype, tp, TYPE_XYZ)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| matfilter(c, duel, fc, sumtype, tp, TYPE_PENDULUM)),
    ];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMTYPE: Option<u32> = Some(1);

    fn group(duel: &Duel, tp: u8) -> CardSet {
        let loc = match duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            false => LOCATION_MZONE | LOCATION_GRAVE,
            true => LOCATION_MZONE,
        };
        duel.get_matching_group(CardId::is_able_to_remove_as_cost, tp, loc, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

// Copies in the Extra Deck, Contact Fused from a Fiend Fusion (D/D/D Flame King Genghis), Synchro, Xyz and Pendulum on the
// field, one in a Pendulum Zone with Dark Contract with the Witch to destroy and Doom Kings in the Deck, and the opponent's
// monsters to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, PZONE),
    (74583607, MZONE),
    (987311, MZONE),
    (46593546, MZONE),
    (72181263, GRAVE),
    (9765723, SZONE),
    (9765723, SZONE),
    (18897163, EXTRA),
    (72402069, EXTRA),
    (26202165, MZONE | OPPONENT),
    (67322708, MZONE | OPPONENT),
    (67322708, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0.set_condition(duel, regcon);
    e0.set_operation(duel, regop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_ATTACK_ALL);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE));
    e4.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e4.set_operation(duel, super::c20715411::penop);
    c.register_effect(duel, e4);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, typ: u32) -> bool {
    c.is_race_full(duel, RACE_FIEND, Some(fc), sumtype, tp) && c.is_type_full(duel, typ, Some(fc), sumtype, tp)
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_fusion_summoned(duel) || c.is_summon_type(duel, &[SPECIAL_SUMMON_PROCEDURE])
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        c.is_original_code(duel, ID)
            && (sumtype & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION || sumtype & SPECIAL_SUMMON_PROCEDURE == SPECIAL_SUMMON_PROCEDURE)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

// The Lua passes an undefined global `dc` to spfilter, so it is always nil.
fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARK_CONTRACT])
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !(c.is_set_card(duel, &[SET_DOOM_KING])
        && c.is_type(duel, TYPE_PENDULUM)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
    {
        return false;
    }
    (c.is_location(duel, LOCATION_DECK) && duel.get_mzone_count(tp, None) > 0)
        || (c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) > 0)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let _ft = duel.get_location_count(tp, LOCATION_MZONE) + duel.get_location_count_from_ex(tp, tp, None, None);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_on_field(duel) && chkc.is_faceup(duel) && desfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| desfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    let ct = duel.get_matching_group_count(filter, tp, LOCATION_ONFIELD, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let ct = u8::try_from(ct).unwrap_or(u8::MAX);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, 0, 1, ct, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn faceup_in_extra(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_location(duel, LOCATION_EXTRA)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    let ct = duel.destroy(tg, REASON_EFFECT)?;
    if ct == 0 {
        return Ok(());
    }
    let mmz_ct = duel.get_location_count(tp, LOCATION_MZONE);
    let emz_ct = duel.get_location_count_from_ex_full(tp, tp, None, None, ZONES_EMZ);
    let linkmz_ct = duel.get_location_count_from_ex(tp, tp, None, None);
    let mut ft = (mmz_ct + emz_ct).min(ct);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_EXTRA, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let rescon = |duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| {
        let in_deck = sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None) as i32;
        let in_extra = sg.filter_count(duel, faceup_in_extra, None) as i32;
        (in_deck <= mmz_ct && in_extra <= linkmz_ct, false)
    };
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &rescon, tp, HINTMSG_SPSUMMON)?;
    if sg.is_empty() {
        return Ok(());
    }
    let (fup, fdown) = sg.split(duel, faceup_in_extra, None);
    let (fdown_main, fdown_ex) = fdown.split(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    let (priority_0, priority_1) = if linkmz_ct < mmz_ct { (fup, fdown_main) } else { (fdown_main, fup) };
    for c in priority_0.into_iter().chain(priority_1).chain(fdown_ex) {
        duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    duel.break_effect()?;
    duel.special_summon_complete()?;
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel
        .get_matching_group(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, None)
        .filter(duel, |c, duel| c.is_can_be_disabled_by_effect(duel, e, true), None);
    if g.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    for tc in g {
        tc.negate_effects(duel, c, 0, false);
    }
    Ok(())
}
