// Majespecter Orthrus - Nue
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 66384688;

pub const LISTED_SERIES: &[u16] = &[SET_MAJESPECTER, SET_DRACOSLAYER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_PENDULUM, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_MAJESPECTER], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

// Majespecter Raccoon - Bunbuku (a Pendulum Monster that goes face-up to the Extra Deck as a Link Material) to Link Summon with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (31991800, MZONE),
    (31991800, MZONE),
    (31991800, MZONE),
    (31991800, MZONE),
    (31991800, DECK),
    (31991800, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MAJESPECTER]) && c.is_type(duel, TYPE_PENDULUM) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tefilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAJESPECTER]) && c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !c.is_set_card(duel, &[SET_MAJESPECTER, SET_DRACOSLAYER]) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| {
        !c.is_original_set_card(duel, &[SET_MAJESPECTER, SET_DRACOSLAYER])
    });
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let mut g = duel.select_matching_card(tp, thfilter, tp, LOCATION_EXTRA, 0, 1, 2, None)?;
    if g.is_empty() || duel.send_to_hand(g, tp, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    g.keep_matching(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    let dg = duel.get_matching_group(tefilter, tp, LOCATION_DECK, 0, None);
    if dg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    let teg = aux::select_unselect_group(duel, dg, 1, 2, &aux::dncheck, tp, HINTMSG_CONFIRM)?;
    if !teg.is_empty() {
        duel.break_effect()?;
        duel.send_to_extra_p(teg, tp, REASON_EFFECT)?;
    }
    Ok(())
}
