package FamilyKernelTypes {
    datatype Data;
    class Occurrence;
    struct Structure;
    assoc struct Association;
    interaction Interaction;
    behavior BehaviorBase;
    behavior BehaviorChild :> BehaviorBase;
    behavior OtherBehavior;
    function Function;
    predicate Predicate;
    metaclass Metadata;
}
