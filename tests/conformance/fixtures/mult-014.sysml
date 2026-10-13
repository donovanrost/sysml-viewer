package ArithmeticGuards {
    attribute zeroDivision : ScalarValues::Integer = 3 / 0;
    attribute fractional : ScalarValues::Real = 3 / 2;
    attribute negative : ScalarValues::Integer = 1 - 3;
    attribute tooLarge : ScalarValues::Integer = 170141183460469231731687303715884105727 + 1;
    attribute left : ScalarValues::Integer = right + 1;
    attribute right : ScalarValues::Integer = left + 1;
    calc def Unbound { in x : ScalarValues::Integer; return y : ScalarValues::Integer = x; }
    attribute unknownCall : ScalarValues::Integer = Unbound();
    part division [zeroDivision];
    part fraction [fractional];
    part belowZero [negative];
    part overflow [tooLarge];
    part cycle [left];
    part call [unknownCall];
}
