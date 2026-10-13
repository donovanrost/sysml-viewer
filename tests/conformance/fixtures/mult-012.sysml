package ProviderGuards {
    attribute unvalued [1] : ScalarValues::Integer;
    attribute left [1] : ScalarValues::Integer = right;
    attribute right [1] : ScalarValues::Integer = left;
    variation attribute choice : ScalarValues::Integer = 3;
    part def Context { attribute size [1] : ScalarValues::Integer = 7; }
    part unvaluedBound [unvalued];
    part cycleBound [left];
    part variationBound [choice];
    part featuredBound [Context::size];
}
