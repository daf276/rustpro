// Dark World Accession
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 65956182;

pub const LISTED_SERIES: &[u16] = &[SET_DARK_WORLD];
// Grapha, Dragon Overlord of Dark World in the Extra Deck, from Grapha, Dragon Lord of Dark World and a DARK monster (Zure,
// Knight of Dark World) in the hand or GY; Beiige, Vanguard of Dark World in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (39552584, EXTRA),
    (39552584, EXTRA),
    (34230233, HAND),
    (34230233, GRAVE),
    (7459013, HAND),
    (7459013, GRAVE),
    (33731070, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(39552584), Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_FIEND)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        (c.is_location(duel, LOCATION_HAND) && c.is_discardable_full(duel, REASON_EFFECT + REASON_FUSION + REASON_MATERIAL, None))
            || (c.is_on_field(duel) && c.is_able_to_remove(duel))
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let check = Some((checkmat as fn(&Duel, CardId, u8, CardSet, CardId) -> bool, e.get_handler(duel)));
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, check);
        }
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), check)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, a.tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
        duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
        Ok(false)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn checkmat(duel: &Duel, _: CardId, _: u8, sg: CardSet, fc: CardId) -> bool {
    fc.is_set_card(duel, &[SET_DARK_WORLD]) || !sg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_HAND), 1, None)
}

fn extraop(duel: &mut Duel, e: EffectId, tc: CardId, tp: u8, sg: &mut CardSet) -> Script<()> {
    let rg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if !rg.is_empty() {
        duel.send_to_grave(rg, REASON_DISCARD + REASON_EFFECT + REASON_FUSION + REASON_MATERIAL)?;
        *sg = sg.difference(rg);
    }
    Fusion::banish_material(duel, e, tc, tp, sg)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_WORLD]) && c.is_monster(duel) && c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_matching_card(tgfilter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, a.e) && duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)? > 0 && c.is_location(duel, LOCATION_HAND)) {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.break_effect()?;
        duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}
