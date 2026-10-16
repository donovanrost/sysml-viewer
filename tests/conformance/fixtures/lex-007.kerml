package NumericKerML {
    feature integer = 00012;
    feature exponent = 3e+2;
    feature spaced = 1 . 25;
    feature leading = .5E+1;
    feature noted = 1 //* 海🚢 */ . // fraction
        25;
    feature precise = 9007199254740993 . 00000000000000000001;
    feature extreme = 1e-9223372036854775808;
    feature fixed[12];
    feature realBound[spaced];
    feature extremeBound[extreme];
}
