// KerML 8.3.3.3 Feature::type, 7.3.2.2 specialization, and 9.1 library identities.
package ScalarIntersectionKerML {
    datatype CountA :> ScalarValues::Real, ScalarValues::Integer;
    datatype CountB :> ScalarValues::Integer, ScalarValues::Real;
    datatype Unclassified;
    feature fractional : ScalarValues::Real, ScalarValues::Integer = 1.5;
    feature reversed : ScalarValues::Integer, ScalarValues::Real = 1.5;
    feature inherited : CountA = 1.5;
    feature inheritedReversed : CountB = 1.5;
    feature incompatible : ScalarValues::String, ScalarValues::Integer = 2;
    feature partlyKnown : Unclassified, ScalarValues::Integer = 1.5;
    alias Whole for ScalarValues::Integer;
    feature throughAlias : Whole = 1.5;
}
