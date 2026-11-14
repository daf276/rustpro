// Gem-Knight Lady Lapis Lazuli
use super::c3113836::{gem_knight, register_splimit};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 47611119;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(99645428), FusionMaterial::Filter(gem_knight)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Gem-Knight Lapis and Garnet; on the field with Garnet in the Deck to
// send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (99645428, HAND),
    (91731841, HAND),
    (91731841, DECK),
    (91731841, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    register_splimit(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DAMAGE + CATEGORY_DECKDES);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn special_summoned_count(duel: &Duel, tp: u8) -> i32 {
    duel.get_matching_group_count(CardId::is_special_summoned, tp, LOCATION_MZONE, LOCATION_MZONE, None) as i32
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(filter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None)
                && duel.is_existing_matching_card(CardId::is_special_summoned, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None),
        );
    }
    let ct = special_summoned_count(duel, tp);
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, ct * 500);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
        let ct = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
        if ct > 0 {
            let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
            let ct = special_summoned_count(duel, tp);
            duel.damage(p, (ct * 500) as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
