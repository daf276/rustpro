// Unchained Soul of Rage
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 67680512;

pub const LISTED_SERIES: &[u16] = &[SET_UNCHAINED_SOUL];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_UNCHAINED_SOUL], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field, with Unchained Abomination and Anguish in the Extra Deck to Link Summon and an opponent's Aruha to summon itself.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (29479265, EXTRA), (93084621, EXTRA), (26236560, MZONE | OPPONENT)];
// Both effects have the same description, so the Link Summon of Anguish stands for the first.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(93084621)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e1.set_target(duel, target::<ID, true>);
    e1.set_operation(duel, operation::<ID, true>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg::<ID>);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn lkfilter<const CODE: u32>(c: CardId, duel: &Duel, mg: CardSet) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && !c.is_code(duel, CODE) && c.is_link_summonable_full(duel, None, Some(mg), 2, 2)
}

fn filter<const CODE: u32, const SPECIAL_SUMMONED: bool>(tc: CardId, duel: &Duel, c: CardId, tp: u8) -> bool {
    let mg: CardSet = [c, tc].into_iter().collect();
    tc.is_faceup(duel)
        && tc.is_can_be_link_material(duel, None, None)
        && (!SPECIAL_SUMMONED || tc.is_special_summoned(duel))
        && duel.is_existing_matching_card(|c, duel| lkfilter::<CODE>(c, duel, mg), tp, LOCATION_EXTRA, 0, 1, None)
}

pub fn target<const CODE: u32, const SPECIAL_SUMMONED: bool>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && filter::<CODE, SPECIAL_SUMMONED>(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |tc, duel| filter::<CODE, SPECIAL_SUMMONED>(tc, duel, c, tp),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(
        tp,
        |tc, duel| filter::<CODE, SPECIAL_SUMMONED>(tc, duel, c, tp),
        tp,
        0,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

pub fn operation<const CODE: u32, const SPECIAL_SUMMONED: bool>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        let mg: CardSet = [c, tc].into_iter().collect();
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| lkfilter::<CODE>(c, duel, mg), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.link_summon_full(tp, sc, None, Some(mg), 2, 2)?;
        }
    }
    Ok(())
}

pub fn thcon(duel: &Duel, a: &Args) -> bool {
    a.r & (REASON_EFFECT + REASON_BATTLE) != 0 && a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD)
}

fn thfilter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND) && !c.is_code(duel, CODE) && c.is_able_to_hand(duel)
}

pub fn thtg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter::<CODE>(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter::<CODE>, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter::<CODE>, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

pub fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
