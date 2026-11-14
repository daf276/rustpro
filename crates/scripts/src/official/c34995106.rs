// Branded in White
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 34995106;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(extraop);

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, mg: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        let eg = duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None);
        let check = Some((fcheck as fn(&Duel, CardId, u8, CardSet, CardId) -> bool, e.get_handler(duel)));
        if !eg.is_empty() && mg.union(eg).is_exists(duel, |c, duel| c.is_code(duel, CARD_ALBAZ), 1, None) {
            return (Some(eg), check);
        }
        (None, check)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, LOCATION_GRAVE as i32);
        Ok(false)
    }
}

// Copies in the hand to discard for Fallen of Albaz's Fusion Summon of Albion the Branded Dragon with the opponent's Elfnote
// Fortuna (LIGHT); Fallen of Albaz and Aluber the Jester of Despia as materials for Despian Quaeritis, also from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (CARD_ALBAZ, HAND),
    (CARD_ALBAZ, HAND),
    (CARD_ALBAZ, HAND),
    (CARD_ALBAZ, MZONE),
    (CARD_ALBAZ, GRAVE),
    (62962630, GRAVE),
    (72272462, EXTRA),
    (87746184, EXTRA),
    (85976588, MZONE | OPPONENT),
    (85976588, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(1170), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    if !sg.is_exists(duel, |c, duel| c.is_race(duel, RACE_DRAGON), 1, None) {
        return false;
    }
    if sg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None) {
        return sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_ALBAZ), 1, None);
    }
    true
}

pub fn extraop(duel: &mut Duel, _: EffectId, _: CardId, _: u8, sg: &mut CardSet) -> Script<()> {
    let rg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if !rg.is_empty() {
        duel.remove(rg, POS_FACEUP, REASON_EFFECT | REASON_MATERIAL | REASON_FUSION)?;
        *sg = sg.difference(rg);
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_reason(duel, REASON_COST)
        && a.re
            .is_some_and(|re| re.is_activated(duel) && re.get_handler(duel).is_code(duel, CARD_ALBAZ))
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_category(duel, CATEGORY_SET);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_range(duel, LOCATION_GRAVE);
        e1.set_count_limit(duel, 1, (ID, 1));
        e1.set_target(duel, settg);
        e1.set_operation(duel, setop);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
