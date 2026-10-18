package StateTransitionTargets {
    state def Driver {
        action outside;
        attribute scalar;
        ref borrowed references outside;
        perform action local references outside;
        transition toScalar first outside then scalar;
        transition toReference first outside then borrowed;
        transition toPerformed first outside then local;
        transition toMissing first outside then local.missing;
    }
}
