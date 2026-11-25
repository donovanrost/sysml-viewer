// KerML 8.3.3.3 Feature::type, 7.3.2.2 specialization, and 9.1 library identities.
package ScalarIdentitySysML {
    attribute def Integer :> ScalarValues::String;
    attribute def Boolean :> ScalarValues::Integer;
    attribute def String :> ScalarValues::Boolean;
    attribute text : Integer = "hello";
    attribute count : Boolean = 2;
    attribute flag : String = true;
    attribute integral : ScalarValues::Real, ScalarValues::Integer = 2;
    alias Whole for ScalarValues::Integer;
    attribute throughAlias : Whole = 3;
}
