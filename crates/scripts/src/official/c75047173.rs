// Favorite Contact
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 75047173;

pub const LISTED_NAMES: &[u32] = &[CARD_NEOS];

struct Params;

impl FusionSummonParams for Params {
    const VALUE: u64 = 0;
    const CHKF: Option<u64> = Some(FUSPROC_NOTFUSION | FUSPROC_LISTEDMATS);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_type(duel, TYPE_FUSION) && c.lists_archetype_as_material(duel, &[SET_HERO])
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_able_to_deck(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let todeck = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_deck);
        let grave = duel.get_matching_group(aux::necro_valley_filter(todeck), tp, LOCATION_GRAVE, 0, None);
        let removed = duel.get_matching_group(aux::faceup_filter(todeck), tp, LOCATION_REMOVED, 0, None);
        (Some(grave.union(removed)), None)
    }

    fn extraop(duel: &mut Duel, _: EffectId, _: CardId, tp: u8, sg: &mut CardSet) -> Script<()> {
        let rg = sg.filter(duel, CardId::is_facedown, None);
        if !rg.is_empty() {
            duel.confirm_cards(1 - tp, rg)?;
        }
        let gyrmg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED), None);
        if !gyrmg.is_empty() {
            duel.hint_selection_full(gyrmg, true);
        }
        duel.send_to_deck(*sg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT + REASON_MATERIAL + REASON_FUSION)?;
        let ct = duel
            .get_operated_group()
            .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
        if ct > 0 {
            duel.sort_deckbottom(tp, tp, ct as u16)?;
        }
        *sg = CardSet::default();
        Ok(())
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, sg: Option<CardSet>, chk: i32) -> Script<()> {
        let neos = |sg: CardSet, duel: &Duel| sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_NEOS), 1, None);
        if let (1, Some(tc), Some(sg)) = (chk, tc, sg)
            && neos(sg, duel)
        {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, aux::stringid(ID, 1));
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CANNOT_TO_DECK);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            // Mirrors the Lua, which changes the registered effect's code after cloning it.
            e1.set_code(duel, EFFECT_CANNOT_TO_HAND);
            tc.register_effect(duel, e2);
        }
        Ok(())
    }
}

// Elemental HERO Flame Wingman in the Extra Deck, Fusion Summoned with Elemental HERO Avian on the field and Burstinatrix in
// the hand, and a banished Elemental HERO Ocean to shuffle in as material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (35809262, EXTRA),
    (21844576, MZONE),
    (58932615, HAND),
    (58932615, GRAVE),
    (37195861, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(35809262)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}
