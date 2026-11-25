// KerML 8.3.3.3 Feature::type, 7.3.2.2 specialization, and 9.1 library identities.
package ScalarIdentityKerML {
    datatype Integer :> ScalarValues::String;
    datatype Boolean :> ScalarValues::Integer;
    datatype String :> ScalarValues::Boolean;
    feature text : Integer = "hello";
    feature count : Boolean = 2;
    feature flag : String = true;
    feature integral : ScalarValues::Real, ScalarValues::Integer = 2;
    alias Whole for ScalarValues::Integer;
    feature throughAlias : Whole = 3;
}
