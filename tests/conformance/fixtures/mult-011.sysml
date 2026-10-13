package ProviderBounds {
    attribute count [1] : ScalarValues::Integer = 3;
    alias renamed for count;
    attribute copy [1] : ScalarValues::Integer = count;
    attribute maximum [1] : ScalarValues::Integer = 6;
    attribute large [1] : ScalarValues::Integer = 9007199254740993;
    attribute zero [1] : ScalarValues::Integer = 0;
    part viaAlias [renamed];
    part chain [copy];
    part ranged [count..maximum];
    part exactLarge [large];
    part empty [zero];
}
