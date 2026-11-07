// Raider's Knight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 28781003;

// Break Sword and Malevolent Scythe, the Rank 3 "The Phantom Knights" Xyz Monsters it Xyz Summons.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (62709239, EXTRA), (78449284, EXTRA)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, rk: u32, pg: CardSet) -> bool {
    c.is_type(duel, TYPE_XYZ)
        && (c.is_set_card(duel, &[SET_RAIDRAPTOR])
            || c.is_set_card(duel, &[SET_XYZ_DRAGON])
            || c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]))
        && c.is_rank(duel, &[rk.saturating_sub(1), rk + 1])
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && (pg.is_empty() || pg.contains(mc))
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok((pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
            && duel.is_existing_matching_card(
                |sc, duel| spfilter(sc, duel, e, tp, c, c.get_rank(duel), pg),
                tp,
                LOCATION_EXTRA,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.is_controler(duel, tp) && !c.is_immune_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let rk = c.get_rank(duel);
        let g = duel.select_matching_card(
            tp,
            |sc, duel| spfilter(sc, duel, e, tp, c, rk, pg),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            1,
            None,
        )?;
        if let Some(sc) = g.get_first() {
            sc.set_material(duel, Some(c.into()));
            duel.overlay(sc, c)?;
            if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
                sc.complete_procedure(duel);
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
                e1.set_code(duel, EVENT_PHASE + PHASE_END);
                e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
                e1.set_count_limit(duel, 1, 0);
                e1.set_label_object_group(duel, Some(sc.into()));
                e1.set_condition(duel, descon);
                e1.set_operation(duel, desop);
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN);
                sc.register_flag_effect_full(
                    duel,
                    ID,
                    RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN,
                    EFFECT_FLAG_CLIENT_HINT,
                    1,
                    0,
                    aux::stringid(ID, 1),
                );
                duel.register_effect(e1, tp);
            }
        }
    }
    Ok(())
}

fn label_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    e.get_label_object_group(duel).and_then(|g| g.get_first())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && label_card(duel, a.e).is_some_and(|sc| sc.has_flag_effect(duel, ID))
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sc) = label_card(duel, a.e) {
        duel.destroy(sc, REASON_EFFECT)?;
    }
    Ok(())
}
