// Gladiator Beast Dareios
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72246674;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_GLADIATOR_BEAST], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// Link Summoned from Gladiator Beast Andal (Level 4), with Hoplomus in the hand, Graveyard and Deck to summon and a monster of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (90582719, MZONE),
    (90582719, MZONE),
    (90582719, MZONE),
    (90582719, MZONE),
    (90582719, GRAVE),
    (4253484, HAND),
    (4253484, DECK),
    (4253484, DECK),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, |duel, a| {
        duel.is_battle_phase() && duel.is_turn_player(1 - a.e.get_handler_player(duel))
    });
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GLADIATOR_BEAST]));
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, opp_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST])
        && c.is_can_be_special_summoned(duel, e, 100, tp, false, false)
        && ((c.is_level_below(duel, 4) && c.is_location(duel, LOCATION_HAND | LOCATION_GRAVE))
            || (opp_chk && c.is_location(duel, LOCATION_DECK)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let opp_chk = duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0;
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp, opp_chk),
                tp,
                LOCATION_HAND | LOCATION_GRAVE | LOCATION_DECK,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        tp,
        (LOCATION_HAND | LOCATION_GRAVE | LOCATION_DECK) as i32,
    );
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let opp_chk = duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0;
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, opp_chk));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE | LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.special_summon(sc, 100, tp, tp, false, false, POS_FACEUP)? > 0
        {
            let code = sc.get_original_code(duel);
            sc.register_flag_effect(duel, code, RESET_EVENT + RESETS_STANDARD_DISABLE, 0, 1);
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_target_range(duel, 0xff, 0xff);
    e1.set_target_card(duel, |duel, _, c| !c.is_set_card(duel, &[SET_GLADIATOR_BEAST]));
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(c.is_controler(duel, e.get_handler_player(duel)))),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}
