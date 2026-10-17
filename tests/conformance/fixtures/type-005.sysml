package KernelFamilyTyping {
    attribute data : FamilyKernelTypes::Data;
    occurrence occurrenceValue : FamilyKernelTypes::Occurrence;
    item structureValue : FamilyKernelTypes::Structure;
    connection association : FamilyKernelTypes::Association;
    flow interactionValue : FamilyKernelTypes::Interaction;
    action behaviorValue : FamilyKernelTypes::BehaviorBase;
    state direct : FamilyKernelTypes::BehaviorBase;
    state multiple : FamilyKernelTypes::BehaviorBase, FamilyKernelTypes::OtherBehavior;
    state related : FamilyKernelTypes::BehaviorBase, FamilyKernelTypes::BehaviorChild;
    exhibit state shown : FamilyKernelTypes::BehaviorChild;
    action def LocalAction;
    state actionTyped : LocalAction;
    calc functionValue : FamilyKernelTypes::Function;
    constraint predicateValue : FamilyKernelTypes::Predicate;
    metadata annotation : FamilyKernelTypes::Metadata;
}
