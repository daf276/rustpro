// Nephthys, the Sacred Preserver
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 50669347;

pub const LISTED_SERIES: &[u16] = &[SET_NEPHTHYS];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_NEPHTHYS], Some(lc), sumtype, tp)
    }
}

// Link Summoned from Nephthys monsters (Hand of Nephthys), with Sacred Phoenix of Nephthys in the Deck to search, in the Graveyard to
// Special Summon with a different name, and a Ritual Spell in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (98446407, MZONE),
    (98446407, MZONE),
    (98446407, MZONE),
    (98446407, MZONE),
    (98446407, GRAVE),
    (61441708, DECK),
    (61441708, DECK),
    (61441708, DECK),
    (61441708, GRAVE),
    (46052429, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[8]) && c.is_race(duel, RACE_WINGEDBEAST) && c.is_able_to_hand(duel)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NEPHTHYS])
        && c.is_monster(duel)
        && c.is_faceup(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_original_code_rule(duel)),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_set_card(duel, &[SET_NEPHTHYS])
        && !c.is_original_code_rule(duel, code)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(deckthfilter, tp, LOCATION_DECK, 0, 1, None);
    let lg = e.get_handler(duel).get_linked_group(duel);
    let b2 = lg.is_exists(duel, |c, duel| desfilter(c, duel, e, tp), 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(lg), 1, tp, 0);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    }
    Ok(false)
}

fn gythfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_spell(duel) && c.is_able_to_hand(duel)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let dc = duel
                .select_matching_card(tp, deckthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
                .get_first();
            if let Some(dc) = dc
                && duel.send_to_hand(dc, PLAYER_NONE, REASON_EFFECT)? > 0
                && dc.is_location(duel, LOCATION_HAND)
            {
                duel.confirm_cards(1 - tp, dc)?;
                duel.shuffle_hand(tp)?;
                let gyfilter = aux::necro_valley_filter(gythfilter);
                if duel.is_existing_matching_card(&gyfilter, tp, LOCATION_GRAVE, 0, 1, None)
                    && duel.select_yes_no(tp, aux::stringid(ID, 3))?
                {
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
                    let g = duel.select_matching_card(tp, &gyfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
                    if !g.is_empty() {
                        duel.hint_selection(g);
                        duel.break_effect()?;
                        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                    }
                }
            }
        }
        2 => {
            let c = e.get_handler(duel);
            if !c.is_relate_to_effect(duel, e) {
                return Ok(());
            }
            let lg = c.get_linked_group(duel);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let dc = lg
                .filter_select(duel, tp, |c, duel| desfilter(c, duel, e, tp), 1, 1, None)?
                .get_first();
            let Some(dc) = dc else { return Ok(()) };
            duel.hint_selection(dc);
            if duel.destroy(dc, REASON_EFFECT)? > 0 {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let code = dc.get_original_code_rule(duel);
                let filter = aux::necro_valley_filter(move |c, duel| spfilter(c, duel, e, tp, code));
                let sc = duel
                    .select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
                    .get_first();
                if let Some(sc) = sc
                    && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?
                {
                    sc.negate_effects(duel, c, 0, false);
                }
                duel.special_summon_complete()?;
            }
        }
        _ => {}
    }
    Ok(())
}
