// Aiwass, Divine Spirit of the Law
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 84288367;
const ALEISTER_THE_INVOKER: u32 = 86120751;

// Aleister the Invoker to search and Summon, Dark Hole to send Red-Eyes Dark Dragoon to the Graveyard with this card there, and
// Invoked Magellanica to Fusion Summon from Aleister and EARTH Tao the Chanter.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ALEISTER_THE_INVOKER, DECK),
    (ALEISTER_THE_INVOKER, DECK),
    (ALEISTER_THE_INVOKER, HAND),
    (46247516, HAND),
    (37818794, MZONE),
    (53129443, HAND),
    (48791583, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_INVOKED])
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ALEISTER]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    let trigger_loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_LOCATION);
    if trigger_loc & LOCATION_MZONE > 0 {
        duel.set_possible_operation_info(0, CATEGORY_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    }
    Ok(false)
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_summonable(duel, true, None)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        let trigger_loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_LOCATION);
        if trigger_loc & LOCATION_MZONE > 0
            && duel.is_existing_matching_card(nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
            if let Some(sc) = duel
                .select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
                .get_first()
            {
                duel.break_effect()?;
                duel.summon(tp, sc, true, None)?;
            }
        }
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_position(duel, POS_FACEUP)
        && c.is_fusion_monster(duel)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, tp)
        && !c.is_reason(duel, REASON_BATTLE)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_FUSION_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        Fusion::new_summon_eff_closure(duel, e, false);
        if Fusion::summon_eff_tg::<Params>(duel, a, 0, None)? && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            duel.break_effect()?;
            Fusion::new_summon_eff_closure(duel, e, true);
            Fusion::summon_eff_op::<Params>(duel, a)?;
        }
    }
    Ok(())
}
