package ConditionalGuards {
    attribute unset : ScalarValues::Boolean;
    attribute unknownTest : ScalarValues::Integer = if unset ? 3 else 4;
    attribute nonBoolean : ScalarValues::Integer = if 1 ? 3 else 4;
    attribute missing : ScalarValues::Integer;
    attribute missingValue : ScalarValues::Integer = if true ? missing else 4;
    attribute zeroDivision : ScalarValues::Integer = if false ? 4 else 3 / 0;
    attribute negative : ScalarValues::Integer = if true ? -1 else 4;
    attribute fractional : ScalarValues::Real = if true ? 3 / 2 else 4;
    attribute tooLarge : ScalarValues::Integer = if true ? 170141183460469231731687303715884105727 + 1 else 4;
    attribute left : ScalarValues::Integer = if true ? right else 4;
    attribute right : ScalarValues::Integer = if false ? 4 else left;
    attribute ineligibleBranch : ScalarValues::Integer = if true ? 5 else left;
    part unavailable [unknownTest];
    part wrongType [nonBoolean];
    part unbound [missingValue];
    part division [zeroDivision];
    part belowZero [negative];
    part fraction [fractional];
    part overflow [tooLarge];
    part recursive [left];
    part ineligible [ineligibleBranch];
}
