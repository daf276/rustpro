// Cosmic Fortress Gol'gar
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 68319538;

pub const LISTED_SERIES: &[u16] = &[SET_ALIEN];
pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_A];

pub const LISTED_NAMES: &[u32] = &[652362];
pub const MATERIAL: &[u32] = &[652362];

// In the Extra Deck, Synchro Summoned with Alien Ammonite (Level 1 Tuner) and Alien Shocktrooper (Level 4), with face-up Spells (Spirit Message L) to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (652362, MZONE),
    (652362, MZONE),
    (97127906, MZONE),
    (97127906, MZONE),
    (30170981, SZONE),
    (30170981, SZONE),
    (30170981, SZONE | OPPONENT),
    (30170981, SZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[652362])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_set_card_full(duel, &[SET_ALIEN], Some(sc), sumtype, tp)
        })
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 16, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tg) = duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS) else {
        return Ok(());
    };
    let rg = tg.filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    duel.send_to_hand(rg, PLAYER_NONE, REASON_EFFECT)?;
    let ct = rg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    let g = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if ct == 0 || g.is_empty() {
        return Ok(());
    }
    for _ in 0..ct {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(tc) = sg.get_first() {
            tc.add_counter(duel, COUNTER_A, 1);
        }
    }
    Ok(())
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_can_remove_counter(tp, 1, 1, COUNTER_A, 2, REASON_COST));
    }
    duel.remove_counter(tp, 1, 1, COUNTER_A, 2, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
