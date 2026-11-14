// Nerva the Power Patron of Creation
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 53589300;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ARTMAGE], Some(fc), sumtype, tp));
    const MINC: i32 = 3;
    const MAXC: i32 = 3;
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Medius the Pure or Special Summoned by Tributing three monsters
// of different Races; on the field with Artmage Diactorus, whose effect it turns into a destruction.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (27184601, MZONE),
    (24094653, HAND),
    (CARD_MEDIUS_THE_PURE, HAND),
    (CARD_MEDIUS_THE_PURE, HAND),
    (CARD_MEDIUS_THE_PURE, HAND),
    (15025844, MZONE),
    (3134241, MZONE),
    (2971090, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    c.add_must_first_be_fusion_summoned(duel);
    c.set_sp_summon_once(duel, ID);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SPSUM_PARAM);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_target_range(duel, POS_FACEUP_DEFENSE, 0);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, _| duel.get_field_group_count(0, LOCATION_FZONE, LOCATION_FZONE) > 0);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, chcon);
    e2.set_target(duel, chtg);
    e2.set_operation(duel, chop);
    c.register_effect(duel, e2);
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (
        duel.get_location_count_from_ex(tp, tp, Some(sg), e.get_handler(duel)) > 0
            && sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64) == sg.len(),
        false,
    )
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let rg = duel.get_release_group(tp);
    aux::select_unselect_group_check(duel, rg, 3, 3, &|duel, sg, _, _| rescon(duel, sg, a.e, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_release_group(tp);
    let mg = aux::select_unselect_group_full(
        duel,
        rg,
        3,
        3,
        &|duel, sg, _, _| rescon(duel, sg, e, tp),
        tp,
        HINTMSG_RELEASE,
        None,
        true,
    )?;
    if mg.len() == 3 {
        e.set_label_object_group(duel, Some(mg));
        return Ok(true);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.release(g, REASON_COST | REASON_MATERIAL)?;
    Ok(())
}

fn chcon(duel: &Duel, a: &Args) -> bool {
    a.rp == a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) && chain::is_triggering_setcode(duel, a.ev as u8, &[SET_ARTMAGE])
}

fn chtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.get_field_group_count(a.tp, 0, LOCATION_ONFIELD) > 0)
}

fn chop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, desop);
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, 0, LOCATION_ONFIELD);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
