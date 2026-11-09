// Snake-Eyes Doomed Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 58071334;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_SNAKE_EYE], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_ILLUSION, Some(fc), sumtype, tp)),
    ];
}

// Special Summoned by sending two monsters from the Spell & Trap Zone, which Snake-Eyes Vengeance Dragon places there, with
// Prevent Rats on the field and in the Graveyard to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (79415624, MZONE),
    (79415624, MZONE),
    (549481, MZONE),
    (549481, GRAVE),
    (549481, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_fusion_summoned(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, hspcon);
    e0.set_target(duel, hsptg);
    e0.set_operation(duel, hspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
}

fn hspfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster_card(duel)
        && c.is_faceup(duel)
        && c.is_able_to_grave_as_cost(duel)
        && c.is_can_be_fusion_material(duel, None, u64::from(SUMMON_TYPE_FUSION))
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count_from_ex(tp, tp, None, c) > 0 && duel.is_existing_matching_card(hspfilter, tp, LOCATION_STZONE, 0, 2, None)
}

fn hsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card_full(tp, hspfilter, tp, LOCATION_STZONE, 0, 2, 2, true, None)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL)?;
    }
    Ok(())
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    let p = c.get_owner(duel);
    c.is_faceup(duel) && duel.get_location_count(p, LOCATION_SZONE) > 0 && !c.is_forbidden(duel) && c.check_unique_on_field(duel, p)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && plfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(plfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    duel.select_target(tp, plfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let p = tc.get_owner(duel);
    if duel.get_location_count(p, LOCATION_SZONE) == 0 {
        duel.send_to_grave_full(tc, REASON_RULE, PLAYER_NONE, PLAYER_NONE)?;
    } else if tc.check_unique_on_field(duel, p) && duel.move_to_field(tc, tp, p, LOCATION_SZONE, POS_FACEUP, tc.is_monster_card(duel))? {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
        e1.set_reset(duel, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
