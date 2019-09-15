//
// Copyright 2019 Tamas Blummer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//!
//! # Shamir's Secret Sharing as defined by SLIP-0039
//! see https://github.com/satoshilabs/slips/blob/master/slip-0039.md
//!
use bitcoin::util::bip158::{BitStreamWriter, BitStreamReader};
use error::Error;
use std::io::Cursor;

#[derive(Debug)]
pub struct ShamirSecretShare {
    pub id: u16,
    pub iteration_exponent: u8,
    pub group_index: u8,
    pub group_threshold: u8,
    pub group_count: u8,
    pub member_index: u8,
    pub member_threshold: u8,
    pub value: Vec<u8>
}

impl ShamirSecretShare {
    pub fn from_mnemonic(mnemonic: &str) -> Result<ShamirSecretShare, Error> {
        let mut buffer = Vec::new();
        let mut writer = BitStreamWriter::new(&mut buffer);
        let mut words = Vec::new();
        for hw in mnemonic.split(' ') {
            if let Ok(w) = WORDS.binary_search(&hw) {
                words.push(w as u16);
                writer.write(w as u64, 10).unwrap();
            }
            else {
                return Err(Error::Mnemonic("invalid word in the key share"));
            }
        }

        if Self::checksum(words.as_slice()) != 1 {
            return Err(Error::Mnemonic("checksum failed"));
        }

        let mut prefix_cursor = Cursor::new(&buffer[.. 5]);
        let mut prefix_reader = BitStreamReader::new(&mut prefix_cursor);
        Ok(ShamirSecretShare {
            id: prefix_reader.read(15).unwrap() as u16,
            iteration_exponent: prefix_reader.read(5).unwrap() as u8,
            group_index: prefix_reader.read(4).unwrap() as u8,
            group_threshold: (prefix_reader.read(4).unwrap() + 1) as u8,
            group_count: (prefix_reader.read(4).unwrap() + 1) as u8,
            member_index: prefix_reader.read(4).unwrap() as u8,
            member_threshold: (prefix_reader.read(4).unwrap() + 1) as u8,
            value: buffer[5..buffer.len() - 4].to_vec()
        })
    }

    pub fn to_mnemonic (&self) -> String {
        let mut words = Vec::new();
        {
            // compile prefix words
            let mut prefix = Vec::new();
            let mut prefix_writer = BitStreamWriter::new(&mut prefix);
            prefix_writer.write(self.id as u64, 15).unwrap();
            prefix_writer.write(self.iteration_exponent as u64, 5).unwrap();
            prefix_writer.write(self.group_index as u64, 4).unwrap();
            prefix_writer.write((self.group_threshold - 1) as u64, 4).unwrap();
            prefix_writer.write((self.group_count - 1) as u64, 4).unwrap();
            prefix_writer.write(self.member_index as u64, 4).unwrap();
            prefix_writer.write((self.member_threshold - 1) as u64, 4).unwrap();
            prefix_writer.flush().unwrap();
            let mut prefix_cursor = Cursor::new(prefix.as_slice());
            let mut prefix_reader = BitStreamReader::new(&mut prefix_cursor);
            words.push(prefix_reader.read(10).unwrap() as u16);
            words.push(prefix_reader.read(10).unwrap() as u16);
            words.push(prefix_reader.read(10).unwrap() as u16);
            words.push(prefix_reader.read(10).unwrap() as u16);
        }
        {
            // append share in words
            let mut share_cursor = Cursor::new(self.value.as_slice());
            let mut share_reader = BitStreamReader::new(&mut share_cursor);
            while let Ok(w) = share_reader.read(10) {
                words.push(w as u16);
            }
            // dummy checksum
            words.push(0);
            words.push(0);
            words.push(0);
        }
        let chk = Self::checksum(words.as_slice()) ^ 1;
        let len = words.len();
        for i in 0..3 {
            words[len - 3 + i] = ((chk >> (10 * (2 - i as u32))) & 1023) as u16;
        }
        // convert to human readable words
        let mut result = String::new();
        for w in words {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(WORDS[w as usize]);
        }
        result
    }

    fn checksum(values: &[u16]) -> u32 {
        let mut chk = 1u32;
        for v in SALT.iter().chain(values.iter()) {
            let b = chk >> 20;
            chk = ((chk & 0xFFFFF) << 10) ^ (*v as u32);
            for i in 0..10 {
                if (b >> i) & 1 != 0 {
                    chk ^= GEN[i as usize];
                }
            }
        }
        chk
    }
}

const GEN :[u32;10] = [
    0xE0E040,
    0x1C1C080,
    0x3838100,
    0x7070200,
    0xE0E0009,
    0x1C0C2412,
    0x38086C24,
    0x3090FC48,
    0x21B1F890,
    0x3F3F120,
];
const SALT :[u16;6] = ['s' as u16, 'h' as u16, 'a' as u16, 'm' as u16, 'i' as u16, 'r' as u16];

mod test {
    use super::ShamirSecretShare;

    #[test]
    pub fn test_checksum () {
        let m = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard";
        ShamirSecretShare::from_mnemonic(m).expect("this should be valid");
        let m =  "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision kidney";
        assert!(ShamirSecretShare::from_mnemonic(m).is_err());
    }
}

const WORDS: [&str; 1024] = [
"academic",
"acid",
"acne",
"acquire",
"acrobat",
"activity",
"actress",
"adapt",
"adequate",
"adjust",
"admit",
"adorn",
"adult",
"advance",
"advocate",
"afraid",
"again",
"agency",
"agree",
"aide",
"aircraft",
"airline",
"airport",
"ajar",
"alarm",
"album",
"alcohol",
"alien",
"alive",
"alpha",
"already",
"alto",
"aluminum",
"always",
"amazing",
"ambition",
"amount",
"amuse",
"analysis",
"anatomy",
"ancestor",
"ancient",
"angel",
"angry",
"animal",
"answer",
"antenna",
"anxiety",
"apart",
"aquatic",
"arcade",
"arena",
"argue",
"armed",
"artist",
"artwork",
"aspect",
"auction",
"august",
"aunt",
"average",
"aviation",
"avoid",
"award",
"away",
"axis",
"axle",
"beam",
"beard",
"beaver",
"become",
"bedroom",
"behavior",
"being",
"believe",
"belong",
"benefit",
"best",
"beyond",
"bike",
"biology",
"birthday",
"bishop",
"black",
"blanket",
"blessing",
"blimp",
"blind",
"blue",
"body",
"bolt",
"boring",
"born",
"both",
"boundary",
"bracelet",
"branch",
"brave",
"breathe",
"briefing",
"broken",
"brother",
"browser",
"bucket",
"budget",
"building",
"bulb",
"bulge",
"bumpy",
"bundle",
"burden",
"burning",
"busy",
"buyer",
"cage",
"calcium",
"camera",
"campus",
"canyon",
"capacity",
"capital",
"capture",
"carbon",
"cards",
"careful",
"cargo",
"carpet",
"carve",
"category",
"cause",
"ceiling",
"center",
"ceramic",
"champion",
"change",
"charity",
"check",
"chemical",
"chest",
"chew",
"chubby",
"cinema",
"civil",
"class",
"clay",
"cleanup",
"client",
"climate",
"clinic",
"clock",
"clogs",
"closet",
"clothes",
"club",
"cluster",
"coal",
"coastal",
"coding",
"column",
"company",
"corner",
"costume",
"counter",
"course",
"cover",
"cowboy",
"cradle",
"craft",
"crazy",
"credit",
"cricket",
"criminal",
"crisis",
"critical",
"crowd",
"crucial",
"crunch",
"crush",
"crystal",
"cubic",
"cultural",
"curious",
"curly",
"custody",
"cylinder",
"daisy",
"damage",
"dance",
"darkness",
"database",
"daughter",
"deadline",
"deal",
"debris",
"debut",
"decent",
"decision",
"declare",
"decorate",
"decrease",
"deliver",
"demand",
"density",
"deny",
"depart",
"depend",
"depict",
"deploy",
"describe",
"desert",
"desire",
"desktop",
"destroy",
"detailed",
"detect",
"device",
"devote",
"diagnose",
"dictate",
"diet",
"dilemma",
"diminish",
"dining",
"diploma",
"disaster",
"discuss",
"disease",
"dish",
"dismiss",
"display",
"distance",
"dive",
"divorce",
"document",
"domain",
"domestic",
"dominant",
"dough",
"downtown",
"dragon",
"dramatic",
"dream",
"dress",
"drift",
"drink",
"drove",
"drug",
"dryer",
"duckling",
"duke",
"duration",
"dwarf",
"dynamic",
"early",
"earth",
"easel",
"easy",
"echo",
"eclipse",
"ecology",
"edge",
"editor",
"educate",
"either",
"elbow",
"elder",
"election",
"elegant",
"element",
"elephant",
"elevator",
"elite",
"else",
"email",
"emerald",
"emission",
"emperor",
"emphasis",
"employer",
"empty",
"ending",
"endless",
"endorse",
"enemy",
"energy",
"enforce",
"engage",
"enjoy",
"enlarge",
"entrance",
"envelope",
"envy",
"epidemic",
"episode",
"equation",
"equip",
"eraser",
"erode",
"escape",
"estate",
"estimate",
"evaluate",
"evening",
"evidence",
"evil",
"evoke",
"exact",
"example",
"exceed",
"exchange",
"exclude",
"excuse",
"execute",
"exercise",
"exhaust",
"exotic",
"expand",
"expect",
"explain",
"express",
"extend",
"extra",
"eyebrow",
"facility",
"fact",
"failure",
"faint",
"fake",
"false",
"family",
"famous",
"fancy",
"fangs",
"fantasy",
"fatal",
"fatigue",
"favorite",
"fawn",
"fiber",
"fiction",
"filter",
"finance",
"findings",
"finger",
"firefly",
"firm",
"fiscal",
"fishing",
"fitness",
"flame",
"flash",
"flavor",
"flea",
"flexible",
"flip",
"float",
"floral",
"fluff",
"focus",
"forbid",
"force",
"forecast",
"forget",
"formal",
"fortune",
"forward",
"founder",
"fraction",
"fragment",
"frequent",
"freshman",
"friar",
"fridge",
"friendly",
"frost",
"froth",
"frozen",
"fumes",
"funding",
"furl",
"fused",
"galaxy",
"game",
"garbage",
"garden",
"garlic",
"gasoline",
"gather",
"general",
"genius",
"genre",
"genuine",
"geology",
"gesture",
"glad",
"glance",
"glasses",
"glen",
"glimpse",
"goat",
"golden",
"graduate",
"grant",
"grasp",
"gravity",
"gray",
"greatest",
"grief",
"grill",
"grin",
"grocery",
"gross",
"group",
"grownup",
"grumpy",
"guard",
"guest",
"guilt",
"guitar",
"gums",
"hairy",
"hamster",
"hand",
"hanger",
"harvest",
"have",
"havoc",
"hawk",
"hazard",
"headset",
"health",
"hearing",
"heat",
"helpful",
"herald",
"herd",
"hesitate",
"hobo",
"holiday",
"holy",
"home",
"hormone",
"hospital",
"hour",
"huge",
"human",
"humidity",
"hunting",
"husband",
"hush",
"husky",
"hybrid",
"idea",
"identify",
"idle",
"image",
"impact",
"imply",
"improve",
"impulse",
"include",
"income",
"increase",
"index",
"indicate",
"industry",
"infant",
"inform",
"inherit",
"injury",
"inmate",
"insect",
"inside",
"install",
"intend",
"intimate",
"invasion",
"involve",
"iris",
"island",
"isolate",
"item",
"ivory",
"jacket",
"jerky",
"jewelry",
"join",
"judicial",
"juice",
"jump",
"junction",
"junior",
"junk",
"jury",
"justice",
"kernel",
"keyboard",
"kidney",
"kind",
"kitchen",
"knife",
"knit",
"laden",
"ladle",
"ladybug",
"lair",
"lamp",
"language",
"large",
"laser",
"laundry",
"lawsuit",
"leader",
"leaf",
"learn",
"leaves",
"lecture",
"legal",
"legend",
"legs",
"lend",
"length",
"level",
"liberty",
"library",
"license",
"lift",
"likely",
"lilac",
"lily",
"lips",
"liquid",
"listen",
"literary",
"living",
"lizard",
"loan",
"lobe",
"location",
"losing",
"loud",
"loyalty",
"luck",
"lunar",
"lunch",
"lungs",
"luxury",
"lying",
"lyrics",
"machine",
"magazine",
"maiden",
"mailman",
"main",
"makeup",
"making",
"mama",
"manager",
"mandate",
"mansion",
"manual",
"marathon",
"march",
"market",
"marvel",
"mason",
"material",
"math",
"maximum",
"mayor",
"meaning",
"medal",
"medical",
"member",
"memory",
"mental",
"merchant",
"merit",
"method",
"metric",
"midst",
"mild",
"military",
"mineral",
"minister",
"miracle",
"mixed",
"mixture",
"mobile",
"modern",
"modify",
"moisture",
"moment",
"morning",
"mortgage",
"mother",
"mountain",
"mouse",
"move",
"much",
"mule",
"multiple",
"muscle",
"museum",
"music",
"mustang",
"nail",
"national",
"necklace",
"negative",
"nervous",
"network",
"news",
"nuclear",
"numb",
"numerous",
"nylon",
"oasis",
"obesity",
"object",
"observe",
"obtain",
"ocean",
"often",
"olympic",
"omit",
"oral",
"orange",
"orbit",
"order",
"ordinary",
"organize",
"ounce",
"oven",
"overall",
"owner",
"paces",
"pacific",
"package",
"paid",
"painting",
"pajamas",
"pancake",
"pants",
"papa",
"paper",
"parcel",
"parking",
"party",
"patent",
"patrol",
"payment",
"payroll",
"peaceful",
"peanut",
"peasant",
"pecan",
"penalty",
"pencil",
"percent",
"perfect",
"permit",
"petition",
"phantom",
"pharmacy",
"photo",
"phrase",
"physics",
"pickup",
"picture",
"piece",
"pile",
"pink",
"pipeline",
"pistol",
"pitch",
"plains",
"plan",
"plastic",
"platform",
"playoff",
"pleasure",
"plot",
"plunge",
"practice",
"prayer",
"preach",
"predator",
"pregnant",
"premium",
"prepare",
"presence",
"prevent",
"priest",
"primary",
"priority",
"prisoner",
"privacy",
"prize",
"problem",
"process",
"profile",
"program",
"promise",
"prospect",
"provide",
"prune",
"public",
"pulse",
"pumps",
"punish",
"puny",
"pupal",
"purchase",
"purple",
"python",
"quantity",
"quarter",
"quick",
"quiet",
"race",
"racism",
"radar",
"railroad",
"rainbow",
"raisin",
"random",
"ranked",
"rapids",
"raspy",
"reaction",
"realize",
"rebound",
"rebuild",
"recall",
"receiver",
"recover",
"regret",
"regular",
"reject",
"relate",
"remember",
"remind",
"remove",
"render",
"repair",
"repeat",
"replace",
"require",
"rescue",
"research",
"resident",
"response",
"result",
"retailer",
"retreat",
"reunion",
"revenue",
"review",
"reward",
"rhyme",
"rhythm",
"rich",
"rival",
"river",
"robin",
"rocky",
"romantic",
"romp",
"roster",
"round",
"royal",
"ruin",
"ruler",
"rumor",
"sack",
"safari",
"salary",
"salon",
"salt",
"satisfy",
"satoshi",
"saver",
"says",
"scandal",
"scared",
"scatter",
"scene",
"scholar",
"science",
"scout",
"scramble",
"screw",
"script",
"scroll",
"seafood",
"season",
"secret",
"security",
"segment",
"senior",
"shadow",
"shaft",
"shame",
"shaped",
"sharp",
"shelter",
"sheriff",
"short",
"should",
"shrimp",
"sidewalk",
"silent",
"silver",
"similar",
"simple",
"single",
"sister",
"skin",
"skunk",
"slap",
"slavery",
"sled",
"slice",
"slim",
"slow",
"slush",
"smart",
"smear",
"smell",
"smirk",
"smith",
"smoking",
"smug",
"snake",
"snapshot",
"sniff",
"society",
"software",
"soldier",
"solution",
"soul",
"source",
"space",
"spark",
"speak",
"species",
"spelling",
"spend",
"spew",
"spider",
"spill",
"spine",
"spirit",
"spit",
"spray",
"sprinkle",
"square",
"squeeze",
"stadium",
"staff",
"standard",
"starting",
"station",
"stay",
"steady",
"step",
"stick",
"stilt",
"story",
"strategy",
"strike",
"style",
"subject",
"submit",
"sugar",
"suitable",
"sunlight",
"superior",
"surface",
"surprise",
"survive",
"sweater",
"swimming",
"swing",
"switch",
"symbolic",
"sympathy",
"syndrome",
"system",
"tackle",
"tactics",
"tadpole",
"talent",
"task",
"taste",
"taught",
"taxi",
"teacher",
"teammate",
"teaspoon",
"temple",
"tenant",
"tendency",
"tension",
"terminal",
"testify",
"texture",
"thank",
"that",
"theater",
"theory",
"therapy",
"thorn",
"threaten",
"thumb",
"thunder",
"ticket",
"tidy",
"timber",
"timely",
"ting",
"tofu",
"together",
"tolerate",
"total",
"toxic",
"tracks",
"traffic",
"training",
"transfer",
"trash",
"traveler",
"treat",
"trend",
"trial",
"tricycle",
"trip",
"triumph",
"trouble",
"true",
"trust",
"twice",
"twin",
"type",
"typical",
"ugly",
"ultimate",
"umbrella",
"uncover",
"undergo",
"unfair",
"unfold",
"unhappy",
"union",
"universe",
"unkind",
"unknown",
"unusual",
"unwrap",
"upgrade",
"upstairs",
"username",
"usher",
"usual",
"valid",
"valuable",
"vampire",
"vanish",
"various",
"vegan",
"velvet",
"venture",
"verdict",
"verify",
"very",
"veteran",
"vexed",
"victim",
"video",
"view",
"vintage",
"violence",
"viral",
"visitor",
"visual",
"vitamins",
"vocal",
"voice",
"volume",
"voter",
"voting",
"walnut",
"warmth",
"warn",
"watch",
"wavy",
"wealthy",
"weapon",
"webcam",
"welcome",
"welfare",
"western",
"width",
"wildlife",
"window",
"wine",
"wireless",
"wisdom",
"withdraw",
"wits",
"wolf",
"woman",
"work",
"worthy",
"wrap",
"wrist",
"writing",
"wrote",
"year",
"yelp",
"yield",
"yoga",
"zero"
];
