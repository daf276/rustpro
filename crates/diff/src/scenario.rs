use carddata::{Card, CardDatabase, TYPE_EXTRA};
use engine::NewCardInfo;
use engine::constants::*;
use rand::Rng;
use rand::seq::IndexedRandom;

const OTHER_CARDS_MAX: usize = 3;
const XYZ_MATERIALS_MAX: usize = 3;

// Cards an effect of the card under test needs, placed with it in the given location, or in a random zone for 0; for
// the same player, or the opponent with OPPONENT.
const SUPPORT: [(u32, &[(u32, u16)]); 31] = [
    // Fusionist from the Extra Deck with its materials, Petit Angel and Mystical Sheep #2, and Polymerization.
    (1641882, &[(1641882, EXTRA), (24094653, HAND), (38142739, HAND), (83464209, HAND)]),
    // Metallizing Parasite - Lunatite on the field with a monster to equip: Elfnote Lucina.
    (7369217, &[(7369217, MZONE), (13597785, MZONE)]),
    // Polymerization's materials for Albion the Branded Dragon in the Elfnote Extra Deck: Fallen of Albaz and a LIGHT
    // monster, Petit Angel.
    (24094653, &[(68468459, HAND), (38142739, HAND)]),
    // Extox Hydra from the Extra Deck, Fusion Summoned by Polymerization, and on the field to attack.
    (26050548, &[(26050548, EXTRA), (24094653, HAND), (26050548, MZONE)]),
    // Gemini Lancer on the field twice, to be Normal Summoned again.
    (26254876, &[(26254876, MZONE), (26254876, MZONE)]),
    // Blizzard Princess in the hand twice, to be Tribute Summoned with a Spellcaster on the field: Elfnote Lucina.
    (28348537, &[(28348537, HAND), (28348537, HAND), (13597785, MZONE)]),
    // Magician of Black Chaos Ritual Summoned by Black Magic Ritual.
    (30208479, &[(76792184, HAND)]),
    // Salamandra in the hand, to be activated on a FIRE monster: Elfnote Lucina.
    (32268901, &[(32268901, HAND), (13597785, MZONE)]),
    // F.A. Dawn Dragster's level-up needs an "F.A." Spell/Trap: F.A. Downforce.
    (33158448, &[(66322203, 0)]),
    // Ritual Raven as the whole tribute for Magician of Black Chaos (DARK), by Contract with the Abyss.
    (34334692, &[(69035382, HAND), (30208479, HAND)]),
    // Megalith Och on the field Ritual Summons the Megalith Och in the hand with itself, which then draws.
    (36849933, &[(36849933, MZONE), (36849933, HAND)]),
    // Drill Driver Vespenato from the Extra Deck, Xyz Summoned with a Rank 4 Xyz Monster on the field: Tornado Dragon.
    (39317553, &[(39317553, EXTRA), (6983839, MZONE)]),
    // Heavy Mech Support Armor on the field with a Machine monster to equip: Mechanicalchaser.
    (39890958, &[(39890958, MZONE), (7359741, MZONE)]),
    // Advanced Ritual Art in the hand Ritual Summons Magician of Black Chaos with Normal Monsters from the Deck: Mystical
    // Elf and Gene-Warped Warwolf (Level 4), Petit Angel (Level 3).
    (
        46052429,
        &[
            (46052429, HAND),
            (30208479, HAND),
            (15025844, DECK),
            (69247929, DECK),
            (38142739, DECK),
        ],
    ),
    // Fiendish Chain set to be activated on the opponent's Effect Monster, Elfnote Fortuna.
    (50078509, &[(50078509, SZONE), (85976588, MZONE | OPPONENT)]),
    // Formula Synchron from the Extra Deck, Synchro Summoned with Mono Synchron and a Level 4 Warrior, Kojikocy.
    (50091196, &[(50091196, EXTRA), (56897896, MZONE), (1184620, MZONE)]),
    // Darklord Desire in the hand twice, to be Tribute Summoned or Set with a Fairy on the field: Medius the Pure.
    (55690251, &[(55690251, HAND), (55690251, HAND), (97556336, MZONE)]),
    // Mono Synchron on the field with a Level 4 Warrior, Kojikocy, for Formula Synchron in the Extra Deck.
    (56897896, &[(50091196, EXTRA), (56897896, MZONE), (1184620, MZONE)]),
    // Magistus Chorozo from the Extra Deck, by its contact fusion, and on the field for its attack trigger.
    (66532962, &[(66532962, EXTRA), (66532962, MZONE)]),
    // Fallen of Albaz to Normal Summon, and the opponent's LIGHT monster for Albion the Branded Dragon: Elfnote Fortuna.
    (68468459, &[(68468459, HAND), (85976588, MZONE | OPPONENT)]),
    // Contract with the Abyss Ritual Summons Magician of Black Chaos (DARK), with Ritual Raven as the whole tribute.
    (69035382, &[(30208479, HAND), (34334692, HAND)]),
    // Vidolium is Special Summoned by returning a Fusion, a Synchro and an Xyz Monster: Albion the Branded Dragon,
    // Chaos Angel and Gauntlet Launcher.
    (70488851, &[(87746184, 0), (22850702, 0), (15561463, 0)]),
    // Demotion in the hand, to be activated on a face-up monster.
    (72575145, &[(72575145, HAND)]),
    // Black Magic Ritual Ritual Summons Magician of Black Chaos, with Ritual Raven as the whole tribute.
    (76792184, &[(30208479, HAND), (34334692, HAND)]),
    // Inaba White Rabbit in the hand, to be Normal Summoned and return in the End Phase.
    (77084837, &[(77084837, HAND)]),
    // Ecclesia and the Dark Dragon summons a Level 4 or lower monster that lists Fallen of Albaz: Guiding Quem.
    (78397661, &[(45883110, 0)]),
    // Albion the Branded Dragon Fusion Summoned by Polymerization, as Polymerization's support does, so that its effect
    // Fusion Summons Magistus Chorozo with Albion and a Spellcaster.
    (87746184, &[(68468459, HAND), (24094653, HAND), (38142739, HAND), (66532962, EXTRA)]),
    // Kuibelt the Blade Dragon from the Extra Deck with Revolution Synchron in the hand and a Level 4 monster, Kojikocy.
    (87837090, &[(87837090, EXTRA), (97682931, HAND), (97682931, HAND), (1184620, MZONE)]),
    // Gaia Dragon, the Thunder Charger can be Xyz Summoned with a Rank 5 or 6 Xyz Monster: Gauntlet Launcher.
    (91949988, &[(15561463, 0)]),
    // Sea Monster of Theseus from the Extra Deck, Fusion Summoned by Polymerization.
    (96334243, &[(96334243, EXTRA), (24094653, HAND)]),
    // Revolution Synchron in the hand, Synchro material for Kuibelt the Blade Dragon with a Level 4 monster, Kojikocy.
    (97682931, &[(97682931, HAND), (97682931, HAND), (87837090, EXTRA), (1184620, MZONE)]),
];

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
        let support = SUPPORT
            .iter()
            .find(|&&(code, _)| code == card)
            .map_or(&[][..], |&(_, support)| support);
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

const HAND: u16 = LOCATION_HAND as u16;
const DECK: u16 = LOCATION_DECK as u16;
const MZONE: u16 = LOCATION_MZONE as u16;
const SZONE: u16 = LOCATION_SZONE as u16;
const GRAVE: u16 = LOCATION_GRAVE as u16;
const REMOVED: u16 = LOCATION_REMOVED as u16;
const EXTRA: u16 = LOCATION_EXTRA as u16;
const OPPONENT: u16 = 0x8000;

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
