// Charmer Quartet in Bloom
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 27519978;

// Wolf x2 for the Link Summon, and Wolf, Great Angus and Neo Bug in the Graveyard to Special Summon in pairs sharing a Type
// or an Attribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (49417509, MZONE),
    (49417509, MZONE),
    (49417509, GRAVE),
    (11813953, GRAVE),
    (16587243, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, _| {
            let g = duel.get_matching_group(|c, duel| c.is_faceup(duel), 0, LOCATION_MZONE, LOCATION_MZONE, None);
            300 * g.get_bin_class_count(duel, |c, duel| c.get_attribute(duel).into()) as i32
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_NO_TURN_RESET | EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_effect_target(duel, e) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spchk(duel: &Duel, sg: CardSet) -> (bool, bool) {
    let t = sg.get_class_count(duel, |c, duel| c.get_race(duel) as i64);
    let a = sg.get_class_count(duel, |c, duel| c.get_attribute(duel).into());
    ((t == 1 && a == sg.len()) || (a == 1 && t == sg.len()), false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| spchk(duel, sg)));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| spchk(duel, sg), tp, HINTMSG_SPSUMMON)?;
    duel.set_target_card(sg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.len() == 2 && duel.special_summon(tg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let mut g = duel.get_operated_group();
        if c.is_relate_to_effect(duel, e) {
            g.insert(c);
        }
        for tc in g {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 1));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
            e1.set_value(
                duel,
                Value::Material(|_, _, _, sumtype, _| {
                    let allowed = [SUMMON_TYPE_FUSION, SUMMON_TYPE_SYNCHRO, SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK];
                    aux::cannotmatfilter(&allowed.map(u64::from), sumtype)
                }),
            );
            e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}
