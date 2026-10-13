package Components {
    private import ScalarValues::*;

    port def Signal;

    part def Device {
        port signal : Signal;
        attribute mass : Real = 1.0;
    }

    part def SpecialDevice :> Device {
        port redefines signal : Signal;
        attribute serial : Integer = 42;
    }
}
