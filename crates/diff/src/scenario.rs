use carddata::{Card, CardDatabase, TYPE_EXTRA};
use engine::NewCardInfo;
use engine::constants::*;
use rand::Rng;
use rand::seq::IndexedRandom;
use scripts::testing::{DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const OTHER_CARDS_MAX: usize = 3;
const XYZ_MATERIALS_MAX: usize = 3;

// A position built as a puzzle builds one: the LP of both players and the cards Debug.AddCard places before the duel
// starts, on top of the decks.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub lp: [u32; 2],
    pub cards: Vec<NewCardInfo>,
}

impl Scenario {
    // The card under test and its support cards for one player, and up to OTHER_CARDS_MAX of `others` for either, each
    // in a random zone its card type can legally occupy (in the location SUPPORT gives, if any); an Xyz Monster in a
    // Monster Zone gets up to XYZ_MATERIALS_MAX of `others` as materials, owned by either player.
    pub fn generate(card: u32, others: &[u32], database: &CardDatabase, rng: &mut impl Rng) -> Self {
        let player = rng.random_range(0..=1);
        let support = scripts::card(card).map_or(&[][..], |script| script.support);
        let mut placed: Vec<(u32, u8, u16)> = std::iter::once((card, 0))
            .chain(support.iter().copied())
            .map(|(code, location)| match location & OPPONENT {
                0 => (code, player, location),
                _ => (code, 1 - player, location & !OPPONENT),
            })
            .collect();
        for _ in 0..rng.random_range(0..=OTHER_CARDS_MAX) {
            placed.extend(others.choose(rng).map(|&code| (code, rng.random_range(0..=1), 0)));
        }
        let mut used = 0;
        let mut cards = Vec::new();
        for (code, player, location) in placed {
            let Some(card) = database.get(code) else { continue };
            let Some(info) = place(card, player, location, &mut used, rng) else {
                continue;
            };
            cards.push(info);
            if card.card_type & TYPE_XYZ != 0 && info.loc == MZONE.into() {
                for _ in 0..rng.random_range(0..=XYZ_MATERIALS_MAX) {
                    cards.extend(others.choose(rng).map(|&code| NewCardInfo {
                        code,
                        team: rng.random_range(0..=1),
                        ..info
                    }));
                }
            }
        }
        let lp = [0, 1].map(|_| if rng.random() { 8000 } else { rng.random_range(1..=80) * 100 });
        Self { lp, cards }
    }

    pub fn lua(&self) -> String {
        let players = (0..2)
            .zip(self.lp)
            .map(|(player, lp)| format!("Debug.SetPlayerInfo({player},{lp},5,1)\n"));
        let cards = self.cards.iter().map(|info| {
            let NewCardInfo {
                code,
                team,
                con,
                loc,
                seq,
                pos,
                ..
            } = *info;
            format!("Debug.AddCard({code},{team},{con},{loc},{seq},{pos},{})\n", proc_complete(info))
        });
        players.chain(cards).collect()
    }

    // Where each card is as messages give it (controller, location, sequence), with its code; the Pendulum and Field
    // Zones are Spell & Trap Zones there.
    pub fn places(&self) -> impl Iterator<Item = ((u8, u8, u32), u32)> {
        self.cards.iter().map(|info| {
            let (location, sequence) = match info.loc as u16 {
                LOCATION_PZONE => (SZONE, info.seq * 4),
                LOCATION_FZONE => (SZONE, 5),
                location => (location, info.seq),
            };
            ((info.con, location as u8, sequence), info.code)
        })
    }

    pub fn place(&self, duel: &mut engine::Duel, database: &'static CardDatabase) -> Result<(), engine::DuelError> {
        self.cards
            .iter()
            .try_for_each(|&info| duel.debug_add_card(info, proc_complete(&info), database))
    }
}

// A monster on the field, in the GY, banished or face-up in the Extra Deck got there with its summon procedure complete
// (OCG_DuelNewCard sets it for the Monster Zone alike); only hand and Deck cards lose it.
fn proc_complete(info: &NewCardInfo) -> bool {
    let location = info.loc as u8;
    location & (LOCATION_HAND | LOCATION_DECK) == 0 && (location != LOCATION_EXTRA || info.pos & POS_FACEUP != 0)
}

// A random legal zone and position for the card, in `location` unless that is 0, None when the chosen kind of zone is
// full. `used` holds the occupied zones as the core's used_location bits, player 1's from bit 32.
fn place(card: &Card, player: u8, location: u16, used: &mut u64, rng: &mut impl Rng) -> Option<NewCardInfo> {
    let card_type = card.card_type;
    let extra = card_type & TYPE_EXTRA != 0;
    let mut options = vec![(GRAVE, POS_FACEUP), (REMOVED, POS_FACEUP), (REMOVED, POS_FACEDOWN)];
    if extra {
        options.push((EXTRA, POS_FACEDOWN_DEFENSE));
    } else {
        options.extend([(HAND, POS_FACEDOWN), (DECK, POS_FACEDOWN_DEFENSE)]);
    }
    if card_type & TYPE_MONSTER != 0 {
        let positions: &[u32] = match card_type {
            t if t & TYPE_LINK != 0 => &[POS_FACEUP_ATTACK],
            _ if extra => &[POS_FACEUP_ATTACK, POS_FACEUP_DEFENSE],
            _ => &[POS_FACEUP_ATTACK, POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE],
        };
        options.extend(positions.iter().map(|&position| (MZONE, position)));
        if card_type & TYPE_PENDULUM != 0 {
            options.extend([(EXTRA, POS_FACEUP), (LOCATION_PZONE, POS_FACEUP)]);
        }
    } else if card_type & TYPE_FIELD != 0 {
        options.extend([(LOCATION_FZONE, POS_FACEUP), (LOCATION_FZONE, POS_FACEDOWN)]);
    } else {
        options.push((SZONE, POS_FACEDOWN));
        if card_type & TYPE_CONTINUOUS != 0 {
            options.push((SZONE, POS_FACEUP));
        }
    }
    options.retain(|&(option, _)| location == 0 || option == location);
    let &(location, position) = options.choose(rng)?;
    let bit = |sequence: u32| match location {
        MZONE => Some(sequence),
        SZONE => Some(8 + sequence),
        LOCATION_PZONE => Some(8 + sequence * 4),
        LOCATION_FZONE => Some(13),
        _ => None,
    };
    let sequences = match location {
        MZONE if extra => 0..7,
        MZONE | SZONE => 0..5,
        LOCATION_PZONE | DECK => 0..2,
        _ => 0..1,
    };
    let offset = 32 * u32::from(player);
    let free: Vec<u32> = sequences
        .filter(|&sequence| bit(sequence).is_none_or(|bit| *used & 1 << (offset + bit) == 0))
        .collect();
    let &sequence = free.choose(rng)?;
    *used |= bit(sequence).map_or(0, |bit| 1 << (offset + bit));
    if location == MZONE && sequence >= 5 {
        // The opponent sees the same Extra Monster Zone from the other side.
        *used |= 1 << (32 - offset + 11 - sequence);
    }
    Some(NewCardInfo {
        team: player,
        duelist: 0,
        code: card.code,
        con: player,
        loc: location.into(),
        seq: sequence,
        pos: position,
    })
}
