package FormulaBounds {
    attribute count : ScalarValues::Integer = 3;
    attribute total : ScalarValues::Integer = count + 2;
    attribute maximum : ScalarValues::Integer = count * 2;
    part def Assembly {
        part exact [total];
        part ranged [count..maximum];
        part open [*];
        part zero [0];
    }
}
