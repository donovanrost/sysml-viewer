package OrderedComparisonGuards {
    attribute missing;
    attribute booleanLeftCount = if true < 3 ? 1 else 2;
    part booleanLeft [booleanLeftCount];
    attribute booleanRightCount = if 3 >= false ? 1 else 2;
    part booleanRight [booleanRightCount];
    attribute realCount = if 1.0 < 2 ? 1 else 2;
    part real [realCount];
    attribute infinityCount = if * > 3 ? 1 else 2;
    part infinity [infinityCount];
    attribute divisionCount = if 1 / 0 < 3 ? 1 else 2;
    part division [divisionCount];
    attribute excessiveCount = if 2 ^ 1000000 > 3 ? 1 else 2;
    part excessive [excessiveCount];
    attribute booleanBoundCount = 2 < 3;
    part booleanBound [booleanBoundCount];
    attribute missingOperandCount = if missing < 3 ? 1 else 2;
    part missingOperand [missingOperandCount];
    attribute selectedFractionCount = if 2 <= 3 ? 3 / 2 else 2;
    part selectedFraction [selectedFractionCount];
}
