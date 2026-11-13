// Ekhajar, Descendant Dragon of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 65424481;

// Synchro Summoned from Georgius, Swordman of the Ice Barrier and a Level 3 Normal Monster (Water Element), with the opponent's
// Sangan and Raigeki to banish and to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32991027, MZONE),
    (32991027, MZONE),
    (3732747, MZONE),
    (3732747, MZONE),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_cost(duel, |duel, a, chk| {
        if chk == 0 {
            return Ok(aux::ice_barrier_discard_cost_check(duel, a.tp, aux::TRUE, true, 1));
        }
        aux::ice_barrier_discard_cost(duel, a.tp, aux::TRUE, true, (1, 1))?;
        Ok(false)
    });
    e1.set_target(duel, |duel, a, chk, _| {
        let tp = a.tp;
        if chk == 0 {
            return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 2, None));
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 2, 1 - tp, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
        Ok(false)
    });
    e1.set_operation(duel, |duel, a| {
        let tp = a.tp;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 2, 2, None)?;
        if g.len() == 2 {
            duel.hint_selection(g);
            duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
        }
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None))
    });
    e2.set_cost(duel, |duel, a, chk| {
        let tp = a.tp;
        if chk == 0 {
            return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, None));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
        duel.remove(g, POS_FACEUP, REASON_COST)?;
        Ok(false)
    });
    e2.set_target(duel, |duel, a, chk, _| {
        let tp = a.tp;
        if chk == 0 {
            return Ok(duel.is_existing_matching_card(
                CardId::is_able_to_hand,
                tp,
                LOCATION_ONFIELD | LOCATION_GRAVE,
                LOCATION_ONFIELD | LOCATION_GRAVE,
                1,
                None,
            ));
        }
        duel.set_operation_info(
            0,
            CATEGORY_REMOVE,
            None,
            1,
            PLAYER_EITHER,
            (LOCATION_ONFIELD | LOCATION_GRAVE) as i32,
        );
        Ok(false)
    });
    e2.set_operation(duel, |duel, a| {
        let tp = a.tp;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let loc = LOCATION_ONFIELD | LOCATION_GRAVE;
        let g = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, loc, loc, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        }
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_synchro_summoned(duel) && c.is_previous_location(duel, LOCATION_MZONE) && duel.is_turn_player(1 - a.tp)
    });
    e3.set_target(duel, |duel, a, chk, _| {
        let tp = a.tp;
        let loc = LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE;
        if chk == 0 {
            return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, loc, 2, None));
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, loc as i32);
        Ok(false)
    });
    e3.set_operation(duel, bangop);
    c.register_effect(duel, e3);
}

fn bangop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let hand_group = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_HAND, None);
    let field_grave_group = duel.get_matching_group(CardId::is_able_to_remove, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, None);
    let (option_1, option_2) = (!hand_group.is_empty(), !field_grave_group.is_empty());
    if !(option_1 || option_2) {
        return Ok(());
    }
    let chosen = if option_1 && option_2 {
        duel.select_effect(tp, &[(option_1, aux::stringid(ID, 3)), (option_2, aux::stringid(ID, 4))])?
    } else {
        None
    };
    let bang = match chosen.or(option_1.then_some(1)).or(option_2.then_some(2)) {
        Some(1) => hand_group.random_select(duel, tp, 1),
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let bang = field_grave_group.select(duel, tp, 1, 1, None)?;
            duel.hint_selection(bang);
            bang
        }
        _ => CardSet::default(),
    };
    if !bang.is_empty() {
        duel.remove(bang, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
