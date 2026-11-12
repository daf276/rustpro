// Firewall Dragon Darkfluid - Neo Tempest Terahertz
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 64211118;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with three Cyberse monsters (Firewall Dragon, Link Disciples and Linkuriboh are Links), with a
// Cyberse monster (Digitron) in the Deck, and the opponent's monsters to attack and negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (32295838, MZONE),
    (32295838, MZONE),
    (32295838, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
    (36211150, DECK),
    (36211150, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        duel.is_battle_phase()
            && a.rp == 1 - a.tp
            && a.re.is_some_and(|re| re.is_monster_effect(duel))
            && duel.is_chain_disablable(a.ev as u8)
    });
    e1.set_operation(duel, |duel, a| {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, aux::stat_change_damage_step_condition);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e3.set_value(
        duel,
        Value::Card(|duel, e, _| {
            let attributes = CardSet::from(e.get_handler(duel)).get_bin_class_count(duel, |c, duel| u64::from(c.get_attribute(duel)));
            attributes as i32 - 1
        }),
    );
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(c.into()), 1, tp, 2500);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let sc = duel
        .select_matching_card(tp, tgfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?
        .first();
    if let Some(sc) = sc
        && duel.send_to_grave(sc, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_GRAVE)
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
    {
        let attribute = sc.get_attribute(duel) as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e1.set_code(duel, EFFECT_ADD_ATTRIBUTE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_value(duel, attribute);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_ATTACK);
        e2.set_value(duel, 2500);
        c.register_effect(duel, e2);
    }
    Ok(())
}
