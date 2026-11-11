// Ancient Gear Ballista
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 10547580;
const GEARTOWN: u32 = 37694547;

pub const LISTED_NAMES: &[u32] = &[GEARTOWN];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_MACHINE, Some(lc), sumtype, tp) && c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Mighty Guard (a Machine Normal Monster, EARTH), with Ancient Gear Box and Geartown in the
// Deck to search, a Set Spell/Trap (Geartown) to destroy and the opponent's monster to weaken.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62327910, MZONE),
    (62327910, MZONE),
    (62327910, MZONE),
    (60953949, DECK),
    (GEARTOWN, DECK),
    (GEARTOWN, SZONE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_ANCIENT_GEAR]) && c.is_monster(duel)) || c.is_code(duel, GEARTOWN)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn atkdeffilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.get_attack(duel) > 0 || c.get_defense(duel) > 0)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.is_existing_target(atkdeffilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.select_target(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g2 = duel.select_target(tp, atkdeffilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g1, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, g2, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let op_cards = |duel: &Duel, category| {
        duel.get_operation_info(0, category)
            .and_then(|info| info.op_cards)
            .unwrap_or_default()
    };
    let (g1, g2) = (op_cards(duel, CATEGORY_DESTROY), op_cards(duel, CATEGORY_ATKCHANGE));
    if g1.get_first().is_some_and(|c| c.is_relate_to_effect(duel, e)) && duel.destroy(g1, REASON_EFFECT)? > 0 {
        let Some(tc) = g2.get_first() else { return Ok(()) };
        if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
            e1.set_value(duel, 0);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
