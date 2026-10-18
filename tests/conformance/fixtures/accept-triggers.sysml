package AcceptTriggers {
    part receiver;
    attribute moment : Time::TimeInstantValue;
    attribute delay : ISQ::DurationValue;
    attribute ready : ScalarValues::Boolean;
    action def Reception {
        action absolute accept eventAt at moment;
        action relative accept eventAfter after delay via receiver;
        action conditional accept eventWhen when ready;
    }
    state def Driver {
        state a;
        state b;
        transition atTime first a accept timePayload at moment then b;
        transition afterDelay first a accept delayPayload after delay via receiver then b;
        transition onCondition first a accept conditionPayload when ready then b;
    }
}
