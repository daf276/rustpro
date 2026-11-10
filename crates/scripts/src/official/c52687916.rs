// Trishula, Dragon of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 52687916;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Synchro Summoned with Angel Trumpeter (a Level 4 Tuner), Key Mace (Level 1) and Sky Scout (Level 4), with the opponent's Sangans
// and a monster in their Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (1929294, MZONE),
    (30532390, MZONE),
    (26202165, MZONE | OPPONENT),
    (73481154, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, remtg);
    e1.set_operation(duel, remop);
    c.register_effect(duel, e1);
}

fn remtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let mut locations = LOCATION_HAND | LOCATION_ONFIELD;
    if duel.is_player_affected_by_effect(1 - tp, CARD_SPIRIT_ELIMINATION).is_none() {
        locations |= LOCATION_GRAVE;
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, locations, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 0, locations as i32);
    Ok(false)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn remop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g1 = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD, None);
    let g2 = duel.get_matching_group(rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, None);
    let g3 = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, None);
    let mut sg = CardSet::default();
    if !g1.is_empty() && ((g2.is_empty() && g3.is_empty()) || duel.select_yes_no(tp, aux::stringid(ID, 1))?) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg1 = g1.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(sg1);
        sg = sg.union(sg1);
    }
    if !g2.is_empty() && ((sg.is_empty() && g3.is_empty()) || duel.select_yes_no(tp, aux::stringid(ID, 2))?) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg2 = g2.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(sg2);
        sg = sg.union(sg2);
    }
    if !g3.is_empty() && (sg.is_empty() || duel.select_yes_no(tp, aux::stringid(ID, 3))?) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        sg = sg.union(g3.random_select(duel, tp, 1));
    }
    duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
