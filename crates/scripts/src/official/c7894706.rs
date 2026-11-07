// The Chaotic Phantasmal Sacred Beasts
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 7894706;
const SPECIAL_SUMMON_PROCEDURE: u32 = SUMMON_TYPE_SPECIAL + 1;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(|c, duel, _, _, _| matfilter(c, duel)); 3];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMTYPE: Option<u32> = Some(1);

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(CardId::is_able_to_grave_as_cost, tp, LOCATION_MZONE, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

// The three Level 10 materials on the field for the Contact Fusion, a copy in the Extra Deck to summon and one on the
// field, and the opponent's monster to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (23856331, MZONE),
    (50251045, MZONE),
    (96345184, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
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
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 2, 0);
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_EFFECT > 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 3, 0);
    e2.set_cost(duel, Cost::soft_once_per_chain::<ID>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[10]) && !c.is_summonable_card(duel)
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

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_negatable_monster(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let tc = g.and_then(|g| g.get_first());
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DISABLE, tc.map(CardSet::from), 1, tp, 0);
    let half = tc.map_or(0, |tc| tc.get_attack(duel).div_euclid(2));
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, tp, half);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.is_can_be_disabled_by_effect(duel, e, true)
    {
        tc.negate_effects(duel, e.get_handler(duel), RESETS_STANDARD_PHASE_END, false);
        duel.adjust_instantly(Some(tc));
        let lp = tc.get_attack(duel).div_euclid(2);
        if lp > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
            duel.break_effect()?;
            duel.recover(p, lp as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
