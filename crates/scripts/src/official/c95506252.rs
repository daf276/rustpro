// Black Jack the Shadow-Armored Knight
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT, SZONE};

const ID: u32 = 95506252;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Copies on the field, and the opponent's Spells and Traps moving into its Spell & Trap Zone.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (83746708, SZONE | OPPONENT), (83746708, SZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_MOVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| {
            eg.is_exists(
                duel,
                |c, duel| c.is_controler(duel, 1 - a.tp) && c.is_location(duel, LOCATION_STZONE),
                1,
                None,
            )
        })
    });
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
    aux::add_er_equip_limit(duel, c, None, e2);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE);
    e3a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_value(duel, Value::Card(|duel, _, c| equip_level_sum(duel, c) as i32 * 300));
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3b);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_code(duel, EFFECT_SELF_DESTROY);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, |duel, a| equip_level_sum(duel, a.e.get_handler(duel)) > 21);
    c.register_effect(duel, e4);
}

fn equip_level_sum(duel: &Duel, c: CardId) -> i64 {
    c.get_equip_group(duel)
        .filter(duel, |c, duel| c.has_level(duel), None)
        .get_sum(duel, |c, duel| i64::from(c.get_level(duel)))
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_EQUIP, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let deck_count = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    if deck_count == 0 {
        return Ok(());
    }
    duel.disable_shuffle_check(true);
    let g = duel.get_matching_group(|c, duel| c.is_monster(duel), tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        duel.confirm_decktop(tp, deck_count)?;
        duel.sort_deckbottom(tp, tp, deck_count as u16)?;
        return Ok(());
    }
    let c = e.get_handler(duel);
    let Some(ec) = g
        .get_max_group(duel, |c, duel| i64::from(c.get_sequence(duel)))
        .and_then(|(g, _)| g.get_first())
    else {
        return Ok(());
    };
    let ec_seq = ec.get_sequence(duel) as usize;
    duel.confirm_decktop(tp, deck_count - ec_seq)?;
    let mut top_g = duel.get_decktop_group(tp, deck_count - ec_seq);
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && duel.get_location_count_full(tp, LOCATION_SZONE, 0, LOCATION_REASON_TOFIELD, 0xff) > 0
        && !ec.is_forbidden(duel)
        && ec.check_unique_on_field(duel, tp)
    {
        c.equip_by_effect_and_limit_register(duel, e, tp, ec, None, Some(true))?;
    } else {
        duel.send_to_grave(ec, REASON_RULE)?;
    }
    top_g.remove(ec);
    duel.move_to_deck_bottom(top_g)?;
    duel.sort_deckbottom(tp, tp, top_g.len() as u16)?;
    Ok(())
}
