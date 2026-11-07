// Clown Crew Dristy
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 30109445;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        let types = TYPE_RITUAL | TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_PENDULUM;
        c.is_type_full(duel, types, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck and on the field, Link Summoned with two Clown Crew Flair (Ritual Monsters), with Clown Crew Biancaviso in the
// hand to Tribute it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, MZONE), (42759961, MZONE), (42759961, MZONE), (82159583, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_tribute_summoned(duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn linkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_link_monster(duel) && c.is_faceup(duel) && c.is_able_to_extra(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID)
        && duel.is_existing_matching_card(
            linkfilter,
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            LOCATION_MZONE | LOCATION_GRAVE,
            1,
            None,
        );
    let b2 = !duel.has_flag_effect(tp, ID + 1)
        && ((duel.is_player_can_draw(tp, 0) && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, None))
            || (duel.is_player_can_draw(1 - tp, 0)
                && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, 0, LOCATION_HAND, 1, None)));
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TOEXTRA);
        duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, PLAYER_ALL, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TODECK | CATEGORY_DRAW);
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_ALL, LOCATION_HAND as i32);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            let g = duel.get_matching_group(
                linkfilter,
                tp,
                LOCATION_MZONE | LOCATION_GRAVE,
                LOCATION_MZONE | LOCATION_GRAVE,
                None,
            );
            if !g.is_empty() {
                duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
        2 => {
            let turn_player = duel.get_turn_player();
            for p in [turn_player, 1 - turn_player] {
                let g = duel.get_field_group(p, LOCATION_HAND, 0);
                if !g.is_empty() && duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0 && duel.is_player_can_draw(p, 0)
                {
                    let draw_count = duel
                        .get_operated_group()
                        .filter_count(duel, |c, duel| c.is_previous_controler(duel, p), None);
                    if draw_count > 0 {
                        duel.shuffle_deck(p)?;
                        duel.break_effect()?;
                        duel.draw(p, draw_count as u32, REASON_EFFECT)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
