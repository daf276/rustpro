// D/D/D Abyss King Gilgamesh
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 9024198;

pub const LISTED_SERIES: &[u16] = &[SET_DD];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_DD], Some(lc), sumtype, tp)
    }
}

// Link Summoned from D/D/D Xyz Monsters, with the opponent's Raigeki to destroy it and a D/D/D in the Graveyard to summon; D/D
// Savant Thomas and D/D/D Chaos King Apocalypse in the Deck to place.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (51497409, MZONE),
    (51497409, MZONE),
    (3758046, MZONE),
    (3758046, MZONE),
    (51497409, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (41546, DECK),
    (41546, DECK),
    (83303851, DECK),
    (83303851, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn pcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DD]) && c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(pcfilter, tp, LOCATION_DECK, 0, None);
        return Ok(duel.check_location(tp, LOCATION_PZONE, 0)
            && duel.check_location(tp, LOCATION_PZONE, 1)
            && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 2);
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 1000);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_DD]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let g = duel.get_matching_group(pcfilter, tp, LOCATION_DECK, 0, None);
    if duel.check_location(tp, LOCATION_PZONE, 0)
        && duel.check_location(tp, LOCATION_PZONE, 1)
        && g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 2
    {
        let pg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_TOFIELD)?;
        if pg.len() != 2 {
            return Ok(());
        }
        let mut cards = pg.into_iter();
        let (Some(pc1), Some(pc2)) = (cards.next(), cards.next()) else {
            return Ok(());
        };
        if duel.move_to_field(pc1, tp, tp, LOCATION_PZONE, POS_FACEUP, false)? {
            if duel.move_to_field(pc2, tp, tp, LOCATION_PZONE, POS_FACEUP, false)? {
                duel.damage(tp, 1000, REASON_EFFECT)?;
                pc2.set_status(duel, STATUS_EFFECT_ENABLED, true);
            }
            pc1.set_status(duel, STATUS_EFFECT_ENABLED, true);
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && (c.is_reason(duel, REASON_EFFECT)
            || (c.is_reason(duel, REASON_BATTLE) && duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - tp))))
        && c.is_link_summoned(duel)
        && a.rp == 1 - tp
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if c.is_location(duel, LOCATION_GRAVE) && duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) <= 0 {
        return false;
    }
    c.is_set_card(duel, &[SET_DD]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_GRAVE | LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_GRAVE | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
