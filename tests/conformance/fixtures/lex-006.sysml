package NumericSysML {
    attribute integer = 00012;
    attribute exponent = 3e+2;
    attribute spaced = 1 . 25;
    attribute leading = .5E+1;
    attribute noted = 1 //* 海🚢 */ . // fraction
        25;
    attribute precise = 9007199254740993 . 00000000000000000001;
    attribute extreme = 1e-9223372036854775808;
    part fixed[12];
    part realBound[spaced];
    part extremeBound[extreme];
}
