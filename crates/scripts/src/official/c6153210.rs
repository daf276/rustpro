// Ketu Dracotail
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 6153210;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON | RACE_SPELLCASTER)
    }
}

// The script lists the archetype where a card name belongs.
pub const LISTED_NAMES: &[u32] = &[SET_DRACOTAIL as u32];

// Dracotail monsters in the Deck to add, and Dracotail Arthalion in the Extra Deck to Fusion Summon with them against the
// opponent's monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (75003700, DECK),
    (75003700, DECK),
    (1498449, HAND),
    (33760966, EXTRA),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(33760966)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON,
    );
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DRACOTAIL]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_deck(tp)?;
        if duel.get_matching_group_count(|_, _| true, tp, 0, LOCATION_MZONE, None) > 0 {
            Fusion::new_summon_eff_closure(duel, e, false);
            if Fusion::summon_eff_tg::<Params>(duel, a, 0, None)? && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
                duel.break_effect()?;
                Fusion::new_summon_eff_closure(duel, e, true);
                Fusion::summon_eff_op::<Params>(duel, a)?;
            }
        }
    }
    Ok(())
}
