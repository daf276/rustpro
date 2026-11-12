// Materiactor Exagard
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72409226;
const EXAREPTOR: u32 = 37426272;

// In the Extra Deck, Xyz Summoned with Level 3 monsters (Materiactor Exareptor, Nekogal #1, Mountain Warrior), with Exareptor in the
// Deck and a Normal Monster in the Graveyard, and the opponent's Normal Summons to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (EXAREPTOR, MZONE),
    (EXAREPTOR, MZONE),
    (1761063, MZONE),
    (4931562, MZONE),
    (EXAREPTOR, DECK),
    (EXAREPTOR, DECK),
    (1761063, GRAVE),
    (1761063, HAND | OPPONENT),
    (4931562, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, deckspthtg);
    e1.set_operation(duel, deckspthop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.unwrap_or_default()
            .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None)
    });
    e2.set_target(duel, xyzthtg);
    e2.set_operation(duel, xyzthop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn deckspthfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    c.is_set_card(duel, &[SET_MATERIACTOR])
        && ((c.is_monster(duel) && mmz_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
            || (c.is_spell_trap(duel) && c.is_able_to_hand(duel)))
}

fn deckspthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(|c, duel| deckspthfilter(c, duel, e, tp, mmz_chk), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn deckspthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let g = duel.select_matching_card(
        tp,
        |c, duel| deckspthfilter(c, duel, e, tp, mmz_chk),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if sc.is_monster(duel) {
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    } else if sc.is_spell_trap(duel) {
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sc)?;
    }
    Ok(())
}

fn xyzthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let og = c.get_overlay_group(duel).filter(duel, CardId::is_able_to_hand, None);
    if chk == 0 {
        return Ok(c.is_type(duel, TYPE_XYZ) && og.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_MATERIACTOR]), 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(og), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn xyzthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && c.is_type(duel, TYPE_XYZ)) {
        return Ok(());
    }
    let og = c.get_overlay_group(duel).filter(duel, CardId::is_able_to_hand, None);
    if og.is_empty() {
        return Ok(());
    }
    let rescon = |duel: &Duel, sg: CardSet, _, _| {
        (
            sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_MATERIACTOR]), 1, None),
            false,
        )
    };
    let sg = aux::select_unselect_group(duel, og, 1, 2, &rescon, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() && duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, sg)?;
        if duel.is_existing_matching_card(|c, duel| c.is_type(duel, TYPE_NORMAL), tp, LOCATION_GRAVE, 0, 1, None)
            && duel.is_existing_matching_card(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
            let rthg = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
            if rthg.is_empty() {
                return Ok(());
            }
            duel.hint_selection(rthg);
            duel.break_effect()?;
            duel.send_to_hand(rthg, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
