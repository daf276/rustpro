// SPYRAL Double Helix
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 1322368;
const SUPER_AGENT: u32 = 41091257;

pub const LISTED_NAMES: &[u32] = &[SUPER_AGENT];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_SPYRAL], Some(lc), sumtype, tp)
    }
}

// SPYRAL Super Agent to Link Summon with and to take from the Deck or Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (SUPER_AGENT, MZONE),
    (SUPER_AGENT, MZONE),
    (SUPER_AGENT, MZONE),
    (SUPER_AGENT, DECK),
    (SUPER_AGENT, DECK),
    (SUPER_AGENT, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, SUPER_AGENT as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zone: u32) -> bool {
    c.is_set_card(duel, &[SET_SPYRAL])
        && c.is_monster(duel)
        && (c.is_able_to_hand(duel) || (zone != 0 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, 0, LOCATION_DECK) > 0
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp, zone),
                tp,
                LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CARDTYPE);
    let opt = duel.select_option(tp, &[70, 71, 72])?;
    e.set_label(duel, i64::from(opt));
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_field_group_count(tp, 0, LOCATION_DECK) == 0 {
        return Ok(());
    }
    duel.confirm_decktop(1 - tp, 1)?;
    let Some(tc) = duel.get_decktop_group(1 - tp, 1).get_first() else {
        return Ok(());
    };
    let opt = e.get_label(duel);
    if (opt == 0 && tc.is_monster(duel)) || (opt == 1 && tc.is_spell(duel)) || (opt == 2 && tc.is_trap(duel)) {
        let zone = e.get_handler(duel).get_linked_zone_full(duel, tp);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, zone));
        let Some(sc) = duel
            .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        if zone != 0 {
            aux::to_hand_or_else(
                duel,
                sc,
                tp,
                |_, duel| {
                    sc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone)
                        && duel.get_location_count(tp, LOCATION_MZONE) > 0
                },
                |duel| duel.special_summon_full(sc, 0, tp, tp, false, false, POS_FACEUP, zone).map(drop),
                aux::stringid(ID, 1),
            )?;
        } else {
            duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sc)?;
        }
    }
    Ok(())
}
