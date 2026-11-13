// Cyberse Contract Witch
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 37458564;
const TYPES_RITUAL_FUSION_SYNCHRO_XYZ: u32 = TYPE_RITUAL | TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_same_property(duel, sg, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp)).0
    }
}

// Link Summoned with Cyberse Normal Monsters, a Spell in the hand to send, and Cyberse Code Magician in the Deck to add and on the
// field, with a Cyberse Synchro Monster (Cyberse Integrator) in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (83764718, HAND),
    (64865, DECK),
    (64865, MZONE),
    (64865, MZONE),
    (64865, MZONE),
    (25200959, GRAVE),
    (25200959, GRAVE),
    (25200959, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1a.set_cost(duel, thcost);
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, |duel, a| {
        aux::zptgroupcon(duel, a.eg.unwrap_or_default(), aux::TRUE, a.e.get_handler(duel))
    });
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND)) && c.is_able_to_grave_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, thcostfilter, LOCATION_HAND | LOCATION_ONFIELD)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, race: u64, card_type: u32) -> bool {
    c.is_type(duel, TYPES_RITUAL_FUSION_SYNCHRO_XYZ)
        && c.is_race(duel, race)
        && !c.is_type(duel, card_type)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lg: CardSet) -> bool {
    c.is_type(duel, TYPES_RITUAL_FUSION_SYNCHRO_XYZ)
        && lg.is_contains(c)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |sc, duel| {
                spfilter(
                    sc,
                    duel,
                    e,
                    tp,
                    c.get_race(duel),
                    c.get_type(duel) & TYPES_RITUAL_FUSION_SYNCHRO_XYZ,
                )
            },
            tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lg = e.get_handler(duel).get_linked_group(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel, e, tp, lg));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| tgfilter(c, duel, e, tp, lg), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(
        tp,
        |c, duel| tgfilter(c, duel, e, tp, lg),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let (race, card_type) = (tc.get_race(duel), tc.get_type(duel) & TYPES_RITUAL_FUSION_SYNCHRO_XYZ);
        let g = duel.select_matching_card(
            tp,
            |c, duel| spfilter(c, duel, e, tp, race, card_type),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
