package IndependentBounds {
    attribute count = 2 + 3;
    attribute selected = if true ? count * 2 else 0;
    alias n for count;
    part exact [n];
    part chosen [selected];
    part range [count..10];
}
package BrokenDomain {
    part def Broken :> MissingBase;
    part def Indirect :> Broken;
    part dependent : Indirect;
}
