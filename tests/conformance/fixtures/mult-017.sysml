package ConditionalProviders {
    attribute enabled : ScalarValues::Boolean = true;
    attribute disabled : ScalarValues::Boolean = false;
    alias flag for enabled;
    attribute missing : ScalarValues::Integer;
    attribute trueCount : ScalarValues::Integer = if flag ? 3 else 4;
    attribute falseCount : ScalarValues::Integer = if disabled ? 3 else 4;
    attribute nested : ScalarValues::Integer = if enabled ? (if disabled ? 9 else 5) else 7;
    attribute formula : ScalarValues::Integer = (if enabled ? 2 + 3 else 8) * 2;
    attribute lazyDivision : ScalarValues::Integer = if enabled ? 6 else 3 / 0;
    attribute lazyUnbound : ScalarValues::Integer = if disabled ? missing else 7;
    attribute exact : ScalarValues::Integer = if enabled ? 9007199254740992 + 1 else 0;
    attribute conditionalTest : ScalarValues::Boolean = if enabled ? false else true;
    attribute selected : ScalarValues::Integer = if conditionalTest ? 2 else 8;
    part trueBranch [trueCount];
    part falseBranch [falseCount];
    part nestedBranch [nested];
    part arithmetic [formula];
    part skippedDivision [lazyDivision];
    part skippedUnbound [lazyUnbound];
    part exactLarge [exact];
    part nestedCondition [selected];
    part range [trueCount..falseCount];
}
