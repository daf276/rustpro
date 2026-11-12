// Number 93: Utopia Kaiser
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 23187256;
pub const XYZ_NUMBER: u32 = 93;

// Number Xyz Monsters of the same Rank on the field and in the Extra Deck, Xyz Summoned over each other with Level 4 Normal
// Monsters, and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (7194917, EXTRA),
    (7194917, EXTRA),
    (63746411, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (2118022, MZONE),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(xyzfilter);
    const LV: Option<u32> = None;
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;

    fn exchk(duel: &Duel, g: CardSet, _: u8, _: CardId) -> bool {
        let mg = g.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_EQUIP_SPELL_XYZ_MAT).is_none(), None);
        mg.get_class_count(duel, |c, duel| i64::from(c.get_rank(duel))) == 1
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, indcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
}

fn xyzfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(xyz), sumtype, tp)
        && c.is_set_card_full(duel, &[SET_NUMBER], Some(xyz), sumtype, tp)
        && c.get_overlay_count(duel) > 0
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rp: u8) -> bool {
    c.is_rank_below(duel, 9)
        && c.is_attack_below(duel, 3000)
        && c.is_set_card(duel, &[SET_NUMBER])
        && duel.get_location_count_from_ex(tp, rp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp, rp), tp, LOCATION_EXTRA, 0, 1, None)
                && e.get_handler(duel).get_overlay_count(duel) > 0,
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn gfilter(c: CardId, duel: &Duel, tp: u8, rp: u8, rank: u32) -> bool {
    !c.is_rank(duel, &[rank]) && duel.get_location_count_from_ex(tp, rp, None, c) > 0
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, rp, c) = (a.e, a.tp, a.rp, a.e.get_handler(duel));
    let mut ct = c
        .get_overlay_group(duel)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    ct = ct.min(aux::check_summon_gate(duel, tp).unwrap_or(ct));
    if ct > 0 {
        let mut g1 = duel.get_matching_group(|c, duel| filter(c, duel, e, tp, rp), tp, LOCATION_EXTRA, 0, None);
        if !g1.is_empty() {
            loop {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let tc = g1.select(duel, tp, 1, 1, None)?.get_first();
                unported!(duel, tc.is_none());
                let Some(tc) = tc else { return Ok(()) };
                duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
                let e1 = duel.create_effect(c);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_code(duel, EFFECT_DISABLE);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e1);
                let e2 = duel.create_effect(c);
                e2.set_type(duel, EFFECT_TYPE_SINGLE);
                e2.set_code(duel, EFFECT_DISABLE_EFFECT);
                e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e2);
                let rank = tc.get_rank(duel);
                g1 = g1.filter(duel, |c, duel| gfilter(c, duel, tp, rp, rank), None);
                ct -= 1;
                if g1.is_empty() || ct == 0 || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
                    break;
                }
            }
            duel.special_summon_complete()?;
            duel.break_effect()?;
            c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
        }
    }
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 0, 1);
    e3.set_value(duel, HALF_DAMAGE as i32);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    e4.set_target_range(duel, 1, 0);
    duel.register_effect(e4, tp);
    Ok(())
}

fn indfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_set_card(duel, &[SET_NUMBER])
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(indfilter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, Some(c.into()))
}
