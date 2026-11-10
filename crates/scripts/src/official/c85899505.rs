// Timaeus the United Magical Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 85899505;

pub const MATERIAL_SETCODE: &[u16] = &[SET_DARK_MAGICIAN, SET_MAGICIAN_GIRL, SET_DARK_MAGICIAN_GIRL];
pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Table(&[
            FusionMaterial::Code(CARD_DARK_MAGICIAN),
            FusionMaterial::Code(CARD_DARK_MAGICIAN_GIRL),
        ]),
        FusionMaterial::Filter(matfilter),
    ];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Dark Magician and Mystical Elf, with Raigeki in
// the Graveyard to count, and the opponent's Sangan to battle and Welcome Home to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (15025844, HAND),
    (12580477, GRAVE),
    (12580477, GRAVE),
    (12580477, GRAVE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (64491754, SZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e0.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0.set_operation(duel, regop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_DRAGON | RACE_SPELLCASTER, Some(fc), sumtype, tp)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ct = if duel.is_turn_player(tp) { 3 } else { 2 };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3100);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(|duel, e, re| e.get_handler(duel) != re.get_owner(duel)));
    e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, ct);
    c.register_effect(duel, e1);
    Ok(())
}

fn spellcount(duel: &Duel, tp: u8) -> usize {
    let filter = aux::faceup_filter(|c, duel| c.is_spell(duel));
    let loc = LOCATION_GRAVE | LOCATION_REMOVED;
    duel.get_matching_group_count(filter, tp, loc, loc, None)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = spellcount(duel, tp);
    if chk == 0 {
        return Ok(ct > 0);
    }
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(a.e.get_handler(duel).into()), 1, tp, 100 * ct as i32);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = spellcount(duel, tp);
    if ct == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 100 * ct as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    // The script indexes the target without a nil check, so a missing one is an error there.
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
