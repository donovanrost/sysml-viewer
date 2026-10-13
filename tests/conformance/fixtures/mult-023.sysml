package CompoundRangeGuards {
    attribute count = 2 + 3;
    attribute missing;
    attribute unavailable = if true ? missing else 10;
    attribute fractional = if true ? 3 / 2 else 10;
    attribute wrongType = if 1 ? 10 else 5;
    attribute selected = if true ? count * 2 else 0;
    part incomplete [count..unavailable];
    part fraction [count..fractional];
    part nonBoolean [count..wrongType];
    part reversed [selected..count];
}
