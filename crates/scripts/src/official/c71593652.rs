// Mutiny in the Sky
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 71593652;

// Fiendsmith's Lacrima in the Extra Deck, Fusion Summoned from two Evil HERO Adusted Gold (LIGHT Fiends) in the GY, with a copy in the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (46640168, EXTRA),
    (13650422, GRAVE),
    (13650422, GRAVE),
    (13650422, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(46640168), Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_FIEND | RACE_FAIRY)
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        (Some(duel.get_matching_group(matfilter, tp, LOCATION_GRAVE, 0, None)), None)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND | RACE_FAIRY) && c.is_able_to_deck(duel)
}

fn thcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FIEND | RACE_FAIRY)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && c.is_able_to_grave_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thcostfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, thcostfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
