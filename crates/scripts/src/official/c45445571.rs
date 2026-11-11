// The Duke of Demise
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 45445571;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(matfilter); 2];
}

// In the Extra Deck and the Graveyard, Fusion Summoned by Polymerization with Skull Servants, with Zombino in the hand to
// Normal Summon and in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, GRAVE),
    (CARD_POLYMERIZATION, HAND),
    (32274490, HAND),
    (32274490, MZONE),
    (14575467, HAND),
    (14575467, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_count_limit(duel, 1, 0);
    e0.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e0.set_operation(duel, maintop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, sump: u8) -> bool {
    c.is_race_full(duel, RACE_FIEND | RACE_ZOMBIE, Some(fc), sumtype, sump)
}

fn maintop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 0, ID.into());
    let b1 = duel.check_lp_cost(tp, 500);
    let op = if b1 {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (true, aux::stringid(ID, 4))])?
    } else {
        Some(2)
    };
    if op == Some(1) {
        duel.pay_lp_cost(tp, 500)?;
    } else if op == Some(2) {
        duel.destroy(a.e.get_handler(duel), REASON_COST)?;
    }
    Ok(())
}

fn summonable(c: CardId, duel: &Duel) -> bool {
    c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_HAND | LOCATION_MZONE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(summonable, tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, summonable, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 4) && c.is_race(duel, RACE_FIEND | RACE_ZOMBIE) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && chkc != c && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
