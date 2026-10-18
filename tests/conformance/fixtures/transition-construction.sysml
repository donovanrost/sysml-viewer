package TransitionConstruction {
    item def Signal { attribute code : ScalarValues::Integer; }
    state def Driver {
        state a;
        state b;
        transition plain first a then b;
        transition triggered first a accept payload : Signal then b;
        transition guarded first a if true then b;
        transition effectful first a do action invoked then b;
        transition full first a accept data : Signal if data.code > 0
            do action work then b {
                in extra;
                out outputData;
                inout both;
            }
        state c;
        transition accept shortPayload : Signal then b;
        state d;
        if true then b;
        state e;
        transition then b;
    }
    action def Decisions {
        action a;
        action b;
        first a if true then b;
    }
}
