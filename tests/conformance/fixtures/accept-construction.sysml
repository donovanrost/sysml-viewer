package AcceptConstruction {
    item def Signal;
    item sample : Signal;
    part receiver;
    action def Reception {
        action simple accept data : Signal;
        action directed accept routed : Signal via receiver;
        action boundValue accept boundPayload : Signal = sample via receiver;
        action bodyParameters accept bodyPayload : Signal {
            in additional = sample;
            out report = sample;
        }
    }
    state def Driver {
        entry accept entering : Signal;
        do accept running : Signal via receiver;
        exit accept leaving : Signal;
        state a;
        state b;
        transition triggered first a accept signal : Signal then b;
    }
}
