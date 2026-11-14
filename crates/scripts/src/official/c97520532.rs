// Gimmick Puppet Chimera Doll
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 97520532;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_MACHINE, Some(lc), sumtype, tp)
    }
}

// Machine Normal Monsters (Cycroid) to Link Summon with, and Gimmick Puppet Scissor Arms in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (45945685, MZONE),
    (45945685, MZONE),
    (45945685, MZONE),
    (45945685, MZONE),
    (29216967, DECK),
    (29216967, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DECKDES + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtgtg);
    e1.set_operation(duel, thtgop);
    c.register_effect(duel, e1);
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_monster(duel) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thtgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    if let Some(sc) = duel
        .select_matching_card(tp, thtgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    {
        let send = |duel: &mut Duel| duel.send_to_grave(sc, REASON_EFFECT).map(|count| count > 0);
        if aux::to_hand_or_else_result(duel, sc, tp, |c, duel| c.is_able_to_grave(duel), send, 574)? {
            if sc.is_location(duel, LOCATION_HAND) {
                duel.shuffle_hand(tp)?;
            }
            let fg = duel.get_field_group(tp, LOCATION_MZONE, 0);
            let hg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
            let puppet = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GIMMICK_PUPPET]));
            if !fg.is_empty()
                && fg.filter_count(duel, puppet, None) == fg.len()
                && !hg.is_empty()
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.select_yes_no(tp, aux::stringid(ID, 2))?
            {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let sg = hg.select(duel, tp, 1, 1, None)?;
                duel.break_effect()?;
                duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !(c.is_type(duel, TYPE_XYZ) && c.is_race(duel, RACE_MACHINE))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| {
        !(c.is_original_type(duel, TYPE_XYZ) && c.is_original_race(duel, RACE_MACHINE))
    });
    Ok(())
}
