package ConditionalBounds {
    attribute count : ScalarValues::Integer = if true ? 3 else 4;
    part sized [count];
}
