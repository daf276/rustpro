// Shaddoll Schism
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, SZONE};

const ID: u32 = 21011044;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];

struct Params;

impl FusionSummonParams for Params {
    const DESC: u64 = aux::stringid(ID, 0);
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_SHADDOLL])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, None);
        }
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None)
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        let Some(tc) = tc.filter(|_| chk == 1) else { return Ok(()) };
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3207);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let attr = tc.get_attribute(duel);
        let tgfilter = |c: CardId, duel: &Duel| c.is_attribute(duel, attr) && c.is_faceup(duel) && c.is_able_to_grave(duel);
        if duel.is_existing_matching_card(tgfilter, tp, 0, LOCATION_MZONE, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
            let g = duel.select_matching_card(tp, tgfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.send_to_grave(g, REASON_EFFECT)?;
            }
        }
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let tp = a.tp;
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, 1 - tp, LOCATION_MZONE as i32);
        Ok(false)
    }
}

// Set, with El Shaddoll Winda in the Extra Deck and Shaddoll Squamata and Dragon in the GY to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (94977269, EXTRA),
    (30328508, GRAVE),
    (77723643, GRAVE),
    (30328508, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(94977269), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    c.register_effect(duel, e1);
}
