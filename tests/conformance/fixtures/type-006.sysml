package ConcernTyping {
    requirement def General;
    concern invalid : General;
    concern def Specific;
    concern def Specialized :> Specific;
    requirement asRequirement : Specific;
    concern valid : Specific;
    concern inherited : Specialized;
    concern redundant : Specific, Specialized;
    concern def Other;
    concern multiple : Specific, Other;
    concern mixed : Specific, General;
}
