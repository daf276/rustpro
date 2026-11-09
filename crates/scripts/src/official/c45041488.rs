// Koa'ki Meiru Guardian
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 45041488;

pub const LISTED_NAMES: &[u32] = &[36623431];

// On the field with Iron Core of Koa'ki Meiru and Koa'ki Meiru Guardian in the hand for its upkeep, and the opponent's Effect Monsters
// to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (36623431, HAND),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_operation(duel, mtop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 3));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, Cost::self_tribute);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn cfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, 36623431) && c.is_able_to_grave_as_cost(duel)
}

fn cfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_race(duel, RACE_ROCK) && !c.is_public(duel)
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint_selection(c);
    let g1 = duel.get_matching_group(cfilter1, tp, LOCATION_HAND, 0, None);
    let g2 = duel.get_matching_group(cfilter2, tp, LOCATION_HAND, 0, None);
    duel.hint(HINT_SELECTMSG, tp, 0);
    let options = [
        (!g1.is_empty(), aux::stringid(ID, 0)),
        (!g2.is_empty(), aux::stringid(ID, 1)),
        (true, aux::stringid(ID, 2)),
    ];
    match duel.select_effect(tp, &options)? {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let g = g1.select(duel, tp, 1, 1, None)?;
            duel.send_to_grave(g, REASON_COST)?;
        }
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let g = g2.select(duel, tp, 1, 1, None)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
        }
        _ => {
            duel.destroy(c, REASON_COST)?;
        }
    }
    Ok(())
}

fn condition(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
