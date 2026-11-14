// Blue-Eyes Spirit Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 59822133;

pub const LISTED_SERIES: &[u16] = &[SET_BLUE_EYES];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Synchro Summoned from Mono Synchron and Blue-Eyes White Dragon, with Blue-Eyes Ultimate Spirit Dragon as
// the Synchro Monster to summon and an opponent's Graveyard effect to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (89604813, EXTRA),
    (56897896, MZONE),
    (97268402, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (73642296, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_set_card_full(duel, &[SET_BLUE_EYES], Some(sc), sumtype, tp)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_condition(duel, negcon);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e3.set_cost(duel, Cost::self_tribute);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.get_activate_location(duel) == LOCATION_GRAVE) && duel.is_chain_negatable(a.ev as u8)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_activation(a.ev as u8);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_type(duel, TYPE_SYNCHRO)
        && duel.get_location_count_from_ex(tp, tp, Some(e.get_handler(duel).into()), c) > 0
        && !c.is_code(duel, ID)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? != 0
    {
        let c = e.get_handler(duel);
        let fid = i64::from(c.get_field_id(duel));
        tc.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, fid, 0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_label(duel, fid);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_condition(duel, descon);
        e1.set_operation(duel, desop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn label_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    e.get_label_object_group(duel).and_then(|g| g.get_first())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    label_card(duel, a.e).is_some_and(|tc| tc.get_flag_effect_label(duel, ID) == Some(a.e.get_label(duel)))
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = label_card(duel, a.e) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
