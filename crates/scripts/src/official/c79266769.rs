// Darkness Metal, the Dragon of Dark Steel
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, REMOVED};

const ID: u32 = 79266769;

// In the Extra Deck, Link Summoned from Sangans (same Type and Attribute), with Sangans in the Graveyard and banished to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, GRAVE),
    (26202165, GRAVE),
    (26202165, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_same_property(duel, sg, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp)).0
            && aux::check_same_property(duel, sg, &|c, duel| u64::from(c.get_attribute_full(duel, Some(lc), sumtype, tp))).0
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp) & ZONES_MMZ;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
            && chkc.is_controler(duel, tp)
            && filter(chkc, duel, e, tp, zone));
    }
    let filter = |c, duel: &Duel| filter(c, duel, e, tp, zone);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    let zone = c.get_linked_zone_full(duel, tp) & ZONES_MMZ;
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && zone != 0
        && duel.special_summon_step_full(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE, zone)?
    {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD));
            tc.register_effect(duel, e1);
        }
        let e3 = duel.create_effect(c);
        e3.set_description(duel, 3301);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e3.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e3.set_value(duel, LOCATION_DECKBOT as i32);
        tc.register_effect(duel, e3);
        let e4 = duel.create_effect(c);
        e4.set_description(duel, aux::stringid(ID, 1));
        e4.set_type(duel, EFFECT_TYPE_FIELD);
        e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e4.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e4.set_target_range(duel, 1, 0);
        e4.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            if c.is_monster(duel) {
                c.is_type(duel, TYPE_LINK)
            } else {
                c.is_original_type(duel, TYPE_LINK)
            }
        });
        e4.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e4, tp);
    }
    duel.special_summon_complete()?;
    Ok(())
}
