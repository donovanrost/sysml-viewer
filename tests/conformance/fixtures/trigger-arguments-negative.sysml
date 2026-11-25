package TriggerWrong {
    attribute def WrongDuration :> Quantities::ScalarQuantityValue {
        attribute :>> mRef : ISQBase::LengthUnit[1];
    }
    attribute def DecoyDuration :> Quantities::ScalarQuantityValue {
        attribute unrelated : ISQBase::DurationUnit;
    }
    attribute wrong : WrongDuration;
    attribute decoy : DecoyDuration;
    attribute length : ISQ::LengthValue;
    attribute scalar : ScalarValues::Real;
    attribute ready : ScalarValues::Boolean;
    action def Reception {
        action wrongUnit accept after wrong;
        action unrelatedUnit accept after decoy;
        action lengthValue accept after length;
        action scalarValue accept after scalar;
        action booleanAfter accept after true;
        action booleanAt accept at ready;
        action integerAt accept at 1;
        action integerWhen accept when 42;
    }
}
