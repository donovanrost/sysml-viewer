package TriggerArguments {
    attribute def CustomDuration :> Quantities::ScalarQuantityValue {
        attribute :>> mRef : ISQBase::DurationUnit[1];
    }
    attribute def InheritedDuration :> CustomDuration;
    attribute def CustomInstant :> Time::TimeInstantValue;
    attribute def CustomBoolean :> ScalarValues::Boolean;
    attribute delay : CustomDuration;
    attribute inheritedDelay : InheritedDuration;
    attribute ordinaryDelay : ISQ::DurationValue;
    attribute moment : CustomInstant;
    attribute ready : CustomBoolean;
    action def Reception {
        action independent accept after delay;
        action inherited accept after inheritedDelay;
        action ordinary accept after ordinaryDelay;
        action absolute accept at moment;
        action conditional accept when ready;
        action literal accept when true;
    }
}
