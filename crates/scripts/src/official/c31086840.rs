// Number 69: Heraldry Crest - Shatter Stream
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 31086840;
pub const XYZ_NUMBER: u32 = 69;
const HERALDRY_CREST: u32 = 2407234;
const DEMOLITION: u32 = 77571454;

// s.listed_series holds these two card codes, which no u16 setcode equals, so Card.ListsArchetype is false for it.
pub const LISTED_SERIES: &[u16] = &[];

// In the Extra Deck, Xyz Summoned with four Level 4 monsters or over Number 69: Heraldry Crest, with Dark Matter Demolition
// in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (DEMOLITION, EXTRA),
    (HERALDRY_CREST, MZONE),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, MZONE),
    (5628232, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(DEMOLITION)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 4;
    const ALTERF: Option<AlterFilter> = Some(|c, duel, _, _| c.is_faceup(duel) && c.is_original_code_rule(duel, HERALDRY_CREST));
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - a.tp))
    });
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let ttype: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_TYPE);
    let tloc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    let tplayer: u8 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_CONTROLER);
    ttype & TYPE_MONSTER > 0 && tloc & LOCATION_ONFIELD > 0 && tplayer == 1 - a.tp
}

fn xyzfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_code(duel, DEMOLITION)
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok((pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
            && duel.is_existing_matching_card(|xc, duel| xyzfilter(xc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None));
    }
    let eg = a.eg.unwrap_or_default();
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(eg), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.is_controler(duel, tp) && !c.is_immune_to_effect(duel, e)) {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if pg.len() > 1 || (pg.len() == 1 && !pg.contains(c)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(xyzc) = duel
        .select_matching_card(tp, |xc, duel| xyzfilter(xc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    xyzc.set_material(duel, Some(c.into()));
    duel.overlay(xyzc, c)?;
    if duel.special_summon(xyzc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
        xyzc.complete_procedure(duel);
        let Some(dg) = duel.get_first_target() else { return Ok(()) };
        if dg.is_controler(duel, 1 - tp)
            && dg.is_relate_to_effect(duel, e)
            && (duel.check_event(EVENT_ATTACK_ANNOUNCE) || a.re.is_some_and(|re| dg.is_relate_to_effect(duel, re)))
        {
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
