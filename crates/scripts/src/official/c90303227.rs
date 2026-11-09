// K9-X "Werewolf"
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 90303227;

// In the Extra Deck for Xyz Summon, and on the field with Xyz Import to attach the opponent's monsters, against an opponent's
// Elfnote Lucina in the hand and on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| (e.get_handler(duel).get_overlay_count(duel) as i32 - 1).max(0)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let loc = if duel.is_turn_player(tp) {
        LOCATION_ONFIELD | LOCATION_GRAVE
    } else {
        LOCATION_HAND
    };
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, 0, loc, 1, None));
    }
    e.set_label(duel, loc.into());
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, loc as i32);
    Ok(false)
}

struct ReturnToHand;

impl aux::DelayedOperation for ReturnToHand {
    fn operate(duel: &mut Duel, rg: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(rg, PLAYER_NONE, REASON_EFFECT)?;
        Ok(())
    }
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let loc = e.get_label(duel) as u32;
    let g = duel.get_field_group(tp, 0, loc);
    if g.is_empty() {
        return Ok(());
    }
    if loc == LOCATION_HAND {
        duel.confirm_cards(tp, g)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let tc = g.filter_select(duel, tp, CardId::is_able_to_remove, 1, 1, None)?;
        aux::remove_until::<ReturnToHand>(duel, tc, POS_FACEUP, REASON_EFFECT | REASON_TEMPORARY, PHASE_END, ID, e, tp)?;
        duel.shuffle_hand(1 - tp)?;
    } else {
        let rescon = |duel: &Duel, sg: CardSet, _, _| {
            let count = |location| sg.filter_count(duel, |c, duel| c.is_location(duel, location), None);
            (count(LOCATION_ONFIELD) <= 1 && count(LOCATION_GRAVE) <= 1, false)
        };
        let rg = aux::select_unselect_group(duel, g, 1, 2, &rescon, tp, HINTMSG_REMOVE)?;
        if rg.is_empty() {
            return Ok(());
        }
        duel.hint_selection(rg);
        duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
