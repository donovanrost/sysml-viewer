package CompoundRanges {
    attribute count = 2 + 3;
    attribute selected = if true ? count * 2 else 0;
    alias limit for selected;
    part exact [count];
    part chosen [selected];
    part range [count..selected];
    part viaAlias [count..limit];
    part repeated [selected..selected];
    part repeatedLower [count..count];
}
