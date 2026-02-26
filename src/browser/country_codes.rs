/// Country dial code dictionary.
/// Maps dial codes to (country name, ISO 3166-1 alpha-2 code).
/// Sorted longest-prefix-first for unambiguous phone number parsing.

pub struct CountryInfo {
    pub dial_code: &'static str,
    pub name: &'static str,
    pub iso: &'static str,
}

/// All country dial codes, sorted longest-first for greedy prefix matching.
static COUNTRIES: &[CountryInfo] = &[
    // 4-digit codes
    CountryInfo {
        dial_code: "1242",
        name: "Bahamas",
        iso: "BS",
    },
    CountryInfo {
        dial_code: "1246",
        name: "Barbados",
        iso: "BB",
    },
    CountryInfo {
        dial_code: "1264",
        name: "Anguilla",
        iso: "AI",
    },
    CountryInfo {
        dial_code: "1268",
        name: "Antigua and Barbuda",
        iso: "AG",
    },
    CountryInfo {
        dial_code: "1284",
        name: "British Virgin Islands",
        iso: "VG",
    },
    CountryInfo {
        dial_code: "1340",
        name: "U.S. Virgin Islands",
        iso: "VI",
    },
    CountryInfo {
        dial_code: "1345",
        name: "Cayman Islands",
        iso: "KY",
    },
    CountryInfo {
        dial_code: "1441",
        name: "Bermuda",
        iso: "BM",
    },
    CountryInfo {
        dial_code: "1473",
        name: "Grenada",
        iso: "GD",
    },
    CountryInfo {
        dial_code: "1649",
        name: "Turks and Caicos Islands",
        iso: "TC",
    },
    CountryInfo {
        dial_code: "1658",
        name: "Jamaica",
        iso: "JM",
    },
    CountryInfo {
        dial_code: "1664",
        name: "Montserrat",
        iso: "MS",
    },
    CountryInfo {
        dial_code: "1670",
        name: "Northern Mariana Islands",
        iso: "MP",
    },
    CountryInfo {
        dial_code: "1671",
        name: "Guam",
        iso: "GU",
    },
    CountryInfo {
        dial_code: "1684",
        name: "American Samoa",
        iso: "AS",
    },
    CountryInfo {
        dial_code: "1721",
        name: "Sint Maarten",
        iso: "SX",
    },
    CountryInfo {
        dial_code: "1758",
        name: "Saint Lucia",
        iso: "LC",
    },
    CountryInfo {
        dial_code: "1767",
        name: "Dominica",
        iso: "DM",
    },
    CountryInfo {
        dial_code: "1784",
        name: "Saint Vincent and the Grenadines",
        iso: "VC",
    },
    CountryInfo {
        dial_code: "1787",
        name: "Puerto Rico",
        iso: "PR",
    },
    CountryInfo {
        dial_code: "1809",
        name: "Dominican Republic",
        iso: "DO",
    },
    CountryInfo {
        dial_code: "1829",
        name: "Dominican Republic",
        iso: "DO",
    },
    CountryInfo {
        dial_code: "1849",
        name: "Dominican Republic",
        iso: "DO",
    },
    CountryInfo {
        dial_code: "1868",
        name: "Trinidad and Tobago",
        iso: "TT",
    },
    CountryInfo {
        dial_code: "1869",
        name: "Saint Kitts and Nevis",
        iso: "KN",
    },
    CountryInfo {
        dial_code: "1876",
        name: "Jamaica",
        iso: "JM",
    },
    CountryInfo {
        dial_code: "1939",
        name: "Puerto Rico",
        iso: "PR",
    },
    // 3-digit codes
    CountryInfo {
        dial_code: "210",
        name: "Western Sahara",
        iso: "EH",
    },
    CountryInfo {
        dial_code: "211",
        name: "South Sudan",
        iso: "SS",
    },
    CountryInfo {
        dial_code: "212",
        name: "Morocco",
        iso: "MA",
    },
    CountryInfo {
        dial_code: "213",
        name: "Algeria",
        iso: "DZ",
    },
    CountryInfo {
        dial_code: "216",
        name: "Tunisia",
        iso: "TN",
    },
    CountryInfo {
        dial_code: "218",
        name: "Libya",
        iso: "LY",
    },
    CountryInfo {
        dial_code: "220",
        name: "Gambia",
        iso: "GM",
    },
    CountryInfo {
        dial_code: "221",
        name: "Senegal",
        iso: "SN",
    },
    CountryInfo {
        dial_code: "222",
        name: "Mauritania",
        iso: "MR",
    },
    CountryInfo {
        dial_code: "223",
        name: "Mali",
        iso: "ML",
    },
    CountryInfo {
        dial_code: "224",
        name: "Guinea",
        iso: "GN",
    },
    CountryInfo {
        dial_code: "225",
        name: "Ivory Coast",
        iso: "CI",
    },
    CountryInfo {
        dial_code: "226",
        name: "Burkina Faso",
        iso: "BF",
    },
    CountryInfo {
        dial_code: "227",
        name: "Niger",
        iso: "NE",
    },
    CountryInfo {
        dial_code: "228",
        name: "Togo",
        iso: "TG",
    },
    CountryInfo {
        dial_code: "229",
        name: "Benin",
        iso: "BJ",
    },
    CountryInfo {
        dial_code: "230",
        name: "Mauritius",
        iso: "MU",
    },
    CountryInfo {
        dial_code: "231",
        name: "Liberia",
        iso: "LR",
    },
    CountryInfo {
        dial_code: "232",
        name: "Sierra Leone",
        iso: "SL",
    },
    CountryInfo {
        dial_code: "233",
        name: "Ghana",
        iso: "GH",
    },
    CountryInfo {
        dial_code: "234",
        name: "Nigeria",
        iso: "NG",
    },
    CountryInfo {
        dial_code: "235",
        name: "Chad",
        iso: "TD",
    },
    CountryInfo {
        dial_code: "236",
        name: "Central African Republic",
        iso: "CF",
    },
    CountryInfo {
        dial_code: "237",
        name: "Cameroon",
        iso: "CM",
    },
    CountryInfo {
        dial_code: "238",
        name: "Cape Verde",
        iso: "CV",
    },
    CountryInfo {
        dial_code: "239",
        name: "São Tomé and Príncipe",
        iso: "ST",
    },
    CountryInfo {
        dial_code: "240",
        name: "Equatorial Guinea",
        iso: "GQ",
    },
    CountryInfo {
        dial_code: "241",
        name: "Gabon",
        iso: "GA",
    },
    CountryInfo {
        dial_code: "242",
        name: "Republic of the Congo",
        iso: "CG",
    },
    CountryInfo {
        dial_code: "243",
        name: "Democratic Republic of the Congo",
        iso: "CD",
    },
    CountryInfo {
        dial_code: "244",
        name: "Angola",
        iso: "AO",
    },
    CountryInfo {
        dial_code: "245",
        name: "Guinea-Bissau",
        iso: "GW",
    },
    CountryInfo {
        dial_code: "246",
        name: "British Indian Ocean Territory",
        iso: "IO",
    },
    CountryInfo {
        dial_code: "247",
        name: "Ascension Island",
        iso: "AC",
    },
    CountryInfo {
        dial_code: "248",
        name: "Seychelles",
        iso: "SC",
    },
    CountryInfo {
        dial_code: "249",
        name: "Sudan",
        iso: "SD",
    },
    CountryInfo {
        dial_code: "250",
        name: "Rwanda",
        iso: "RW",
    },
    CountryInfo {
        dial_code: "251",
        name: "Ethiopia",
        iso: "ET",
    },
    CountryInfo {
        dial_code: "252",
        name: "Somalia",
        iso: "SO",
    },
    CountryInfo {
        dial_code: "253",
        name: "Djibouti",
        iso: "DJ",
    },
    CountryInfo {
        dial_code: "254",
        name: "Kenya",
        iso: "KE",
    },
    CountryInfo {
        dial_code: "255",
        name: "Tanzania",
        iso: "TZ",
    },
    CountryInfo {
        dial_code: "256",
        name: "Uganda",
        iso: "UG",
    },
    CountryInfo {
        dial_code: "257",
        name: "Burundi",
        iso: "BI",
    },
    CountryInfo {
        dial_code: "258",
        name: "Mozambique",
        iso: "MZ",
    },
    CountryInfo {
        dial_code: "260",
        name: "Zambia",
        iso: "ZM",
    },
    CountryInfo {
        dial_code: "261",
        name: "Madagascar",
        iso: "MG",
    },
    CountryInfo {
        dial_code: "262",
        name: "Réunion",
        iso: "RE",
    },
    CountryInfo {
        dial_code: "263",
        name: "Zimbabwe",
        iso: "ZW",
    },
    CountryInfo {
        dial_code: "264",
        name: "Namibia",
        iso: "NA",
    },
    CountryInfo {
        dial_code: "265",
        name: "Malawi",
        iso: "MW",
    },
    CountryInfo {
        dial_code: "266",
        name: "Lesotho",
        iso: "LS",
    },
    CountryInfo {
        dial_code: "267",
        name: "Botswana",
        iso: "BW",
    },
    CountryInfo {
        dial_code: "268",
        name: "Eswatini",
        iso: "SZ",
    },
    CountryInfo {
        dial_code: "269",
        name: "Comoros",
        iso: "KM",
    },
    CountryInfo {
        dial_code: "290",
        name: "Saint Helena",
        iso: "SH",
    },
    CountryInfo {
        dial_code: "291",
        name: "Eritrea",
        iso: "ER",
    },
    CountryInfo {
        dial_code: "297",
        name: "Aruba",
        iso: "AW",
    },
    CountryInfo {
        dial_code: "298",
        name: "Faroe Islands",
        iso: "FO",
    },
    CountryInfo {
        dial_code: "299",
        name: "Greenland",
        iso: "GL",
    },
    CountryInfo {
        dial_code: "350",
        name: "Gibraltar",
        iso: "GI",
    },
    CountryInfo {
        dial_code: "351",
        name: "Portugal",
        iso: "PT",
    },
    CountryInfo {
        dial_code: "352",
        name: "Luxembourg",
        iso: "LU",
    },
    CountryInfo {
        dial_code: "353",
        name: "Ireland",
        iso: "IE",
    },
    CountryInfo {
        dial_code: "354",
        name: "Iceland",
        iso: "IS",
    },
    CountryInfo {
        dial_code: "355",
        name: "Albania",
        iso: "AL",
    },
    CountryInfo {
        dial_code: "356",
        name: "Malta",
        iso: "MT",
    },
    CountryInfo {
        dial_code: "357",
        name: "Cyprus",
        iso: "CY",
    },
    CountryInfo {
        dial_code: "358",
        name: "Finland",
        iso: "FI",
    },
    CountryInfo {
        dial_code: "359",
        name: "Bulgaria",
        iso: "BG",
    },
    CountryInfo {
        dial_code: "370",
        name: "Lithuania",
        iso: "LT",
    },
    CountryInfo {
        dial_code: "371",
        name: "Latvia",
        iso: "LV",
    },
    CountryInfo {
        dial_code: "372",
        name: "Estonia",
        iso: "EE",
    },
    CountryInfo {
        dial_code: "373",
        name: "Moldova",
        iso: "MD",
    },
    CountryInfo {
        dial_code: "374",
        name: "Armenia",
        iso: "AM",
    },
    CountryInfo {
        dial_code: "375",
        name: "Belarus",
        iso: "BY",
    },
    CountryInfo {
        dial_code: "376",
        name: "Andorra",
        iso: "AD",
    },
    CountryInfo {
        dial_code: "377",
        name: "Monaco",
        iso: "MC",
    },
    CountryInfo {
        dial_code: "378",
        name: "San Marino",
        iso: "SM",
    },
    CountryInfo {
        dial_code: "380",
        name: "Ukraine",
        iso: "UA",
    },
    CountryInfo {
        dial_code: "381",
        name: "Serbia",
        iso: "RS",
    },
    CountryInfo {
        dial_code: "382",
        name: "Montenegro",
        iso: "ME",
    },
    CountryInfo {
        dial_code: "383",
        name: "Kosovo",
        iso: "XK",
    },
    CountryInfo {
        dial_code: "385",
        name: "Croatia",
        iso: "HR",
    },
    CountryInfo {
        dial_code: "386",
        name: "Slovenia",
        iso: "SI",
    },
    CountryInfo {
        dial_code: "387",
        name: "Bosnia and Herzegovina",
        iso: "BA",
    },
    CountryInfo {
        dial_code: "389",
        name: "North Macedonia",
        iso: "MK",
    },
    CountryInfo {
        dial_code: "420",
        name: "Czech Republic",
        iso: "CZ",
    },
    CountryInfo {
        dial_code: "421",
        name: "Slovakia",
        iso: "SK",
    },
    CountryInfo {
        dial_code: "423",
        name: "Liechtenstein",
        iso: "LI",
    },
    CountryInfo {
        dial_code: "500",
        name: "Falkland Islands",
        iso: "FK",
    },
    CountryInfo {
        dial_code: "501",
        name: "Belize",
        iso: "BZ",
    },
    CountryInfo {
        dial_code: "502",
        name: "Guatemala",
        iso: "GT",
    },
    CountryInfo {
        dial_code: "503",
        name: "El Salvador",
        iso: "SV",
    },
    CountryInfo {
        dial_code: "504",
        name: "Honduras",
        iso: "HN",
    },
    CountryInfo {
        dial_code: "505",
        name: "Nicaragua",
        iso: "NI",
    },
    CountryInfo {
        dial_code: "506",
        name: "Costa Rica",
        iso: "CR",
    },
    CountryInfo {
        dial_code: "507",
        name: "Panama",
        iso: "PA",
    },
    CountryInfo {
        dial_code: "508",
        name: "Saint Pierre and Miquelon",
        iso: "PM",
    },
    CountryInfo {
        dial_code: "509",
        name: "Haiti",
        iso: "HT",
    },
    CountryInfo {
        dial_code: "590",
        name: "Guadeloupe",
        iso: "GP",
    },
    CountryInfo {
        dial_code: "591",
        name: "Bolivia",
        iso: "BO",
    },
    CountryInfo {
        dial_code: "592",
        name: "Guyana",
        iso: "GY",
    },
    CountryInfo {
        dial_code: "593",
        name: "Ecuador",
        iso: "EC",
    },
    CountryInfo {
        dial_code: "594",
        name: "French Guiana",
        iso: "GF",
    },
    CountryInfo {
        dial_code: "595",
        name: "Paraguay",
        iso: "PY",
    },
    CountryInfo {
        dial_code: "596",
        name: "Martinique",
        iso: "MQ",
    },
    CountryInfo {
        dial_code: "597",
        name: "Suriname",
        iso: "SR",
    },
    CountryInfo {
        dial_code: "598",
        name: "Uruguay",
        iso: "UY",
    },
    CountryInfo {
        dial_code: "599",
        name: "Curaçao",
        iso: "CW",
    },
    CountryInfo {
        dial_code: "670",
        name: "Timor-Leste",
        iso: "TL",
    },
    CountryInfo {
        dial_code: "672",
        name: "Norfolk Island",
        iso: "NF",
    },
    CountryInfo {
        dial_code: "673",
        name: "Brunei",
        iso: "BN",
    },
    CountryInfo {
        dial_code: "674",
        name: "Nauru",
        iso: "NR",
    },
    CountryInfo {
        dial_code: "675",
        name: "Papua New Guinea",
        iso: "PG",
    },
    CountryInfo {
        dial_code: "676",
        name: "Tonga",
        iso: "TO",
    },
    CountryInfo {
        dial_code: "677",
        name: "Solomon Islands",
        iso: "SB",
    },
    CountryInfo {
        dial_code: "678",
        name: "Vanuatu",
        iso: "VU",
    },
    CountryInfo {
        dial_code: "679",
        name: "Fiji",
        iso: "FJ",
    },
    CountryInfo {
        dial_code: "680",
        name: "Palau",
        iso: "PW",
    },
    CountryInfo {
        dial_code: "681",
        name: "Wallis and Futuna",
        iso: "WF",
    },
    CountryInfo {
        dial_code: "682",
        name: "Cook Islands",
        iso: "CK",
    },
    CountryInfo {
        dial_code: "683",
        name: "Niue",
        iso: "NU",
    },
    CountryInfo {
        dial_code: "685",
        name: "Samoa",
        iso: "WS",
    },
    CountryInfo {
        dial_code: "686",
        name: "Kiribati",
        iso: "KI",
    },
    CountryInfo {
        dial_code: "687",
        name: "New Caledonia",
        iso: "NC",
    },
    CountryInfo {
        dial_code: "688",
        name: "Tuvalu",
        iso: "TV",
    },
    CountryInfo {
        dial_code: "689",
        name: "French Polynesia",
        iso: "PF",
    },
    CountryInfo {
        dial_code: "690",
        name: "Tokelau",
        iso: "TK",
    },
    CountryInfo {
        dial_code: "691",
        name: "Micronesia",
        iso: "FM",
    },
    CountryInfo {
        dial_code: "692",
        name: "Marshall Islands",
        iso: "MH",
    },
    CountryInfo {
        dial_code: "850",
        name: "North Korea",
        iso: "KP",
    },
    CountryInfo {
        dial_code: "852",
        name: "Hong Kong",
        iso: "HK",
    },
    CountryInfo {
        dial_code: "853",
        name: "Macau",
        iso: "MO",
    },
    CountryInfo {
        dial_code: "855",
        name: "Cambodia",
        iso: "KH",
    },
    CountryInfo {
        dial_code: "856",
        name: "Laos",
        iso: "LA",
    },
    CountryInfo {
        dial_code: "880",
        name: "Bangladesh",
        iso: "BD",
    },
    CountryInfo {
        dial_code: "886",
        name: "Taiwan",
        iso: "TW",
    },
    CountryInfo {
        dial_code: "960",
        name: "Maldives",
        iso: "MV",
    },
    CountryInfo {
        dial_code: "961",
        name: "Lebanon",
        iso: "LB",
    },
    CountryInfo {
        dial_code: "962",
        name: "Jordan",
        iso: "JO",
    },
    CountryInfo {
        dial_code: "963",
        name: "Syria",
        iso: "SY",
    },
    CountryInfo {
        dial_code: "964",
        name: "Iraq",
        iso: "IQ",
    },
    CountryInfo {
        dial_code: "965",
        name: "Kuwait",
        iso: "KW",
    },
    CountryInfo {
        dial_code: "966",
        name: "Saudi Arabia",
        iso: "SA",
    },
    CountryInfo {
        dial_code: "967",
        name: "Yemen",
        iso: "YE",
    },
    CountryInfo {
        dial_code: "968",
        name: "Oman",
        iso: "OM",
    },
    CountryInfo {
        dial_code: "970",
        name: "Palestine",
        iso: "PS",
    },
    CountryInfo {
        dial_code: "971",
        name: "United Arab Emirates",
        iso: "AE",
    },
    CountryInfo {
        dial_code: "972",
        name: "Israel",
        iso: "IL",
    },
    CountryInfo {
        dial_code: "973",
        name: "Bahrain",
        iso: "BH",
    },
    CountryInfo {
        dial_code: "974",
        name: "Qatar",
        iso: "QA",
    },
    CountryInfo {
        dial_code: "975",
        name: "Bhutan",
        iso: "BT",
    },
    CountryInfo {
        dial_code: "976",
        name: "Mongolia",
        iso: "MN",
    },
    CountryInfo {
        dial_code: "977",
        name: "Nepal",
        iso: "NP",
    },
    CountryInfo {
        dial_code: "992",
        name: "Tajikistan",
        iso: "TJ",
    },
    CountryInfo {
        dial_code: "993",
        name: "Turkmenistan",
        iso: "TM",
    },
    CountryInfo {
        dial_code: "994",
        name: "Azerbaijan",
        iso: "AZ",
    },
    CountryInfo {
        dial_code: "995",
        name: "Georgia",
        iso: "GE",
    },
    CountryInfo {
        dial_code: "996",
        name: "Kyrgyzstan",
        iso: "KG",
    },
    CountryInfo {
        dial_code: "998",
        name: "Uzbekistan",
        iso: "UZ",
    },
    // 2-digit codes
    CountryInfo {
        dial_code: "20",
        name: "Egypt",
        iso: "EG",
    },
    CountryInfo {
        dial_code: "27",
        name: "South Africa",
        iso: "ZA",
    },
    CountryInfo {
        dial_code: "30",
        name: "Greece",
        iso: "GR",
    },
    CountryInfo {
        dial_code: "31",
        name: "Netherlands",
        iso: "NL",
    },
    CountryInfo {
        dial_code: "32",
        name: "Belgium",
        iso: "BE",
    },
    CountryInfo {
        dial_code: "33",
        name: "France",
        iso: "FR",
    },
    CountryInfo {
        dial_code: "34",
        name: "Spain",
        iso: "ES",
    },
    CountryInfo {
        dial_code: "36",
        name: "Hungary",
        iso: "HU",
    },
    CountryInfo {
        dial_code: "39",
        name: "Italy",
        iso: "IT",
    },
    CountryInfo {
        dial_code: "40",
        name: "Romania",
        iso: "RO",
    },
    CountryInfo {
        dial_code: "41",
        name: "Switzerland",
        iso: "CH",
    },
    CountryInfo {
        dial_code: "43",
        name: "Austria",
        iso: "AT",
    },
    CountryInfo {
        dial_code: "44",
        name: "United Kingdom",
        iso: "GB",
    },
    CountryInfo {
        dial_code: "45",
        name: "Denmark",
        iso: "DK",
    },
    CountryInfo {
        dial_code: "46",
        name: "Sweden",
        iso: "SE",
    },
    CountryInfo {
        dial_code: "47",
        name: "Norway",
        iso: "NO",
    },
    CountryInfo {
        dial_code: "48",
        name: "Poland",
        iso: "PL",
    },
    CountryInfo {
        dial_code: "49",
        name: "Germany",
        iso: "DE",
    },
    CountryInfo {
        dial_code: "51",
        name: "Peru",
        iso: "PE",
    },
    CountryInfo {
        dial_code: "52",
        name: "Mexico",
        iso: "MX",
    },
    CountryInfo {
        dial_code: "53",
        name: "Cuba",
        iso: "CU",
    },
    CountryInfo {
        dial_code: "54",
        name: "Argentina",
        iso: "AR",
    },
    CountryInfo {
        dial_code: "55",
        name: "Brazil",
        iso: "BR",
    },
    CountryInfo {
        dial_code: "56",
        name: "Chile",
        iso: "CL",
    },
    CountryInfo {
        dial_code: "57",
        name: "Colombia",
        iso: "CO",
    },
    CountryInfo {
        dial_code: "58",
        name: "Venezuela",
        iso: "VE",
    },
    CountryInfo {
        dial_code: "60",
        name: "Malaysia",
        iso: "MY",
    },
    CountryInfo {
        dial_code: "61",
        name: "Australia",
        iso: "AU",
    },
    CountryInfo {
        dial_code: "62",
        name: "Indonesia",
        iso: "ID",
    },
    CountryInfo {
        dial_code: "63",
        name: "Philippines",
        iso: "PH",
    },
    CountryInfo {
        dial_code: "64",
        name: "New Zealand",
        iso: "NZ",
    },
    CountryInfo {
        dial_code: "65",
        name: "Singapore",
        iso: "SG",
    },
    CountryInfo {
        dial_code: "66",
        name: "Thailand",
        iso: "TH",
    },
    CountryInfo {
        dial_code: "81",
        name: "Japan",
        iso: "JP",
    },
    CountryInfo {
        dial_code: "82",
        name: "South Korea",
        iso: "KR",
    },
    CountryInfo {
        dial_code: "84",
        name: "Vietnam",
        iso: "VN",
    },
    CountryInfo {
        dial_code: "86",
        name: "China",
        iso: "CN",
    },
    CountryInfo {
        dial_code: "90",
        name: "Turkey",
        iso: "TR",
    },
    CountryInfo {
        dial_code: "91",
        name: "India",
        iso: "IN",
    },
    CountryInfo {
        dial_code: "92",
        name: "Pakistan",
        iso: "PK",
    },
    CountryInfo {
        dial_code: "93",
        name: "Afghanistan",
        iso: "AF",
    },
    CountryInfo {
        dial_code: "94",
        name: "Sri Lanka",
        iso: "LK",
    },
    CountryInfo {
        dial_code: "95",
        name: "Myanmar",
        iso: "MM",
    },
    CountryInfo {
        dial_code: "98",
        name: "Iran",
        iso: "IR",
    },
    // 1-digit codes
    CountryInfo {
        dial_code: "1",
        name: "United States",
        iso: "US",
    },
    CountryInfo {
        dial_code: "7",
        name: "Russia",
        iso: "RU",
    },
];

/// Look up country info by dial code (without +).
/// E.g., `lookup_by_code("91")` → Some(India)
pub fn lookup_by_code(code: &str) -> Option<&'static CountryInfo> {
    COUNTRIES.iter().find(|c| c.dial_code == code)
}

/// Parse a phone number string into (dial_code_with_plus, national_number, country_info).
/// Uses greedy longest-prefix match for accuracy.
/// E.g., "+919501005734" → ("+91", "9501005734", Some(India))
pub fn parse_phone(phone: &str) -> (&'static CountryInfo, String) {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    // Try longest prefix first (4, 3, 2, 1 digits)
    for len in (1..=4).rev() {
        if digits.len() >= len {
            let prefix = &digits[..len];
            if let Some(info) = lookup_by_code(prefix) {
                return (info, digits[len..].to_string());
            }
        }
    }

    // Fallback: assume 1-digit code (shouldn't happen with complete dictionary)
    static UNKNOWN: CountryInfo = CountryInfo {
        dial_code: "",
        name: "Unknown",
        iso: "XX",
    };
    (&UNKNOWN, digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_india() {
        let (info, national) = parse_phone("+919501005734");
        assert_eq!(info.dial_code, "91");
        assert_eq!(info.name, "India");
        assert_eq!(national, "9501005734");
    }

    #[test]
    fn test_us() {
        let (info, national) = parse_phone("+19876543210");
        assert_eq!(info.dial_code, "1");
        assert_eq!(info.name, "United States");
        assert_eq!(national, "9876543210");
    }

    #[test]
    fn test_uk() {
        let (info, national) = parse_phone("+447911123456");
        assert_eq!(info.dial_code, "44");
        assert_eq!(info.name, "United Kingdom");
        assert_eq!(national, "7911123456");
    }

    #[test]
    fn test_uae() {
        let (info, national) = parse_phone("+971501234567");
        assert_eq!(info.dial_code, "971");
        assert_eq!(info.name, "United Arab Emirates");
        assert_eq!(national, "501234567");
    }

    #[test]
    fn test_bahamas_nanp() {
        let (info, national) = parse_phone("+12425551234");
        assert_eq!(info.dial_code, "1242");
        assert_eq!(info.name, "Bahamas");
        assert_eq!(national, "5551234");
    }

    #[test]
    fn test_russia() {
        let (info, national) = parse_phone("+79161234567");
        assert_eq!(info.dial_code, "7");
        assert_eq!(info.name, "Russia");
        assert_eq!(national, "9161234567");
    }

    #[test]
    fn test_myanmar() {
        let (info, national) = parse_phone("+959123456");
        assert_eq!(info.dial_code, "95");
        assert_eq!(info.name, "Myanmar");
        assert_eq!(national, "9123456");
    }

    #[test]
    fn test_lookup_by_code() {
        let india = lookup_by_code("91").unwrap();
        assert_eq!(india.name, "India");
        assert_eq!(india.iso, "IN");
    }
}
