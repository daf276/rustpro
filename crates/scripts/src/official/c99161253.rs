// Primite Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 99161253;

pub const LISTED_SERIES: &[u16] = &[SET_PRIMITE];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON)
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, fextrafil);
        let locations = LOCATION_ONFIELD | LOCATION_GRAVE | LOCATION_REMOVED;
        (
            Some(duel.get_matching_group(filter, tp, locations, 0, None)),
            Some((fcheck, e.get_handler(duel))),
        )
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let locations = LOCATION_ONFIELD | LOCATION_GRAVE | LOCATION_REMOVED;
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, a.tp, locations as i32);
        Ok(false)
    }
}

// In the hand with Primite Dragon Nether Berzelius in the Extra Deck, Ether Beryl (Primite) on the field and Mystical Elf
// (Normal Monster) in the GY as materials, and in the GY with Primite Imperial Dragon (Level 6) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (26462013, EXTRA),
    (63198739, MZONE),
    (15025844, GRAVE),
    (15025844, GRAVE),
    (ID, GRAVE),
    (81418467, DECK),
    (81418467, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(26462013), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn fextrafil(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_deck(duel) && (c.is_on_field(duel) || c.is_faceup(duel))
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.is_exists(duel, |c, duel| c.is_type(duel, TYPE_NORMAL), 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 5) && c.is_set_card(duel, &[SET_PRIMITE]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_DECK | LOCATION_GRAVE)
}
