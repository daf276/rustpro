// G Golem Crystal Heart
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 61668670;
const COUNTER_CRYSTAL: u16 = 0x20c;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// On the field and in the Extra Deck with Cyberse Normal Monsters (Digitron, Leotron) to link, and EARTH Link Monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (32295838, MZONE),
    (47226949, MZONE),
    (97661969, GRAVE),
    (49202162, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_CRYSTAL);
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = colinked_effect(duel, c, EFFECT_UPDATE_ATTACK);
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| i32::from(e.get_handler(duel).get_counter(duel, COUNTER_CRYSTAL)) * 600),
    );
    c.register_effect(duel, e2);
    let e3 = colinked_effect(duel, c, EFFECT_EXTRA_ATTACK);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = colinked_effect(duel, c, EFFECT_PIERCE);
    c.register_effect(duel, e4);
}

fn colinked_effect(duel: &mut Duel, c: CardId, code: u32) -> EffectId {
    let e = duel.create_effect(c);
    e.set_type(duel, EFFECT_TYPE_FIELD);
    e.set_code(duel, code);
    e.set_range(duel, LOCATION_MZONE);
    e.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e.set_target_card(duel, ecfilter);
    e
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_link_monster(duel)
        && c.is_attribute(duel, ATTRIBUTE_EARTH)
        && zone != 0
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_free_linked_zone(duel) & ZONES_MMZ;
    let filter = |c: CardId, duel: &Duel| spfilter(c, duel, e, tp, zone);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let zone = c.get_free_linked_zone(duel) & ZONES_MMZ;
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e))
        && zone != 0
        && duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)? > 0
    {
        c.add_counter(duel, COUNTER_CRYSTAL, 1);
    }
    Ok(())
}

fn ecfilter(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_handler(duel).get_mutual_linked_group(duel).is_contains(c) && c.is_attribute(duel, ATTRIBUTE_EARTH)
}
