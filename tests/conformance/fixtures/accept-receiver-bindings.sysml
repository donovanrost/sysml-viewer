package AcceptReceivers {
    item def Signal;
    item sample : Signal;
    part receiver;
    attribute moment : Time::TimeInstantValue;
    attribute delay : ISQ::DurationValue;
    attribute ready : ScalarValues::Boolean;
    action def Reception {
        action plain accept payload : Signal;
        action value accept payload : Signal = sample;
        action absolute accept eventAt at moment {
            in extra = sample;
        }
        action relative accept eventAfter after delay via receiver;
        action conditional accept eventWhen when ready;
    }
    state def Driver {
        entry accept entering when ready;
        do accept running after delay via receiver;
        exit accept leaving at moment;
        state a;
        state b;
        transition atTime first a accept timePayload at moment then b;
        transition afterDelay first a accept delayPayload after delay via receiver then b;
        transition onCondition first a accept conditionPayload when ready then b;
    }
}
