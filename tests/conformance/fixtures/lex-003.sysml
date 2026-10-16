package InvalidLexicalSysML {
    attribute crEscape = "海🚢\r";
    attribute unicodeEscape = "海🚢\海";
    attribute tab = "海🚢	";
    attribute line = "海🚢
";
    attribute c1 = "海🚢";
    part recovered[5];
}
