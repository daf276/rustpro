// Keel, Shipwright Fur Hire
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 66736884;
const FANDORA: u32 = 64400161;

pub const LISTED_NAMES: &[u32] = &[FANDORA];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp))
    }
}

// In the Extra Deck, Link Summoned from Fur Hire monsters of different Types, with Fandora in the Deck to Set and Fur Hire monsters in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (28004531, MZONE),
    (93850652, MZONE),
    (47647354, MZONE),
    (64400161, DECK),
    (64400161, DECK),
    (93850652, HAND),
    (28004531, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_spell_trap(duel)) || c.is_code(duel, FANDORA)) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_FUR_HIRE]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.remove(c, 0, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_REMOVED)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        let filter = |c: CardId, duel: &Duel| spfilter(c, duel, e, tp);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g1 = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g1.is_empty()
            && duel.special_summon(g1, 0, tp, tp, false, false, POS_FACEUP)? > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None)
        {
            let atkg = duel
                .get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None)
                .get_max_group(duel, |c, duel| i64::from(c.get_base_attack(duel)));
            unported!(duel, atkg.is_none());
            let Some((atkg, _)) = atkg else { return Ok(()) };
            if !atkg.is_exists(duel, |c, duel| c.is_controler(duel, 1 - tp), 1, None) {
                return Ok(());
            }
            if duel.select_yes_no(tp, aux::stringid(ID, 3))? {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let g2 = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
                if !g2.is_empty() {
                    duel.break_effect()?;
                    duel.special_summon(g2, 0, tp, tp, false, false, POS_FACEUP)?;
                }
            }
        }
    }
    Ok(())
}
