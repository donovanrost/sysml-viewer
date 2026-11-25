// KerML 8.3.3.3 Feature::type, 7.3.2.2 specialization, and 9.1 library identities.
package ScalarIntersectionSysML {
    attribute def CountA :> ScalarValues::Real, ScalarValues::Integer;
    attribute def CountB :> ScalarValues::Integer, ScalarValues::Real;
    attribute def Unclassified;
    attribute fractional : ScalarValues::Real, ScalarValues::Integer = 1.5;
    attribute reversed : ScalarValues::Integer, ScalarValues::Real = 1.5;
    attribute inherited : CountA = 1.5;
    attribute inheritedReversed : CountB = 1.5;
    attribute incompatible : ScalarValues::String, ScalarValues::Integer = 2;
    attribute partlyKnown : Unclassified, ScalarValues::Integer = 1.5;
    alias Whole for ScalarValues::Integer;
    attribute throughAlias : Whole = 1.5;
}
