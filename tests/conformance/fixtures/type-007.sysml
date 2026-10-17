package StateTypingGuards {
    attribute def Data;
    action def BehaviorType;
    state bad : Data;
    exhibit state badExhibit : Data;
    state mixed : BehaviorType, Data;
    state good : BehaviorType;
    state missing : Absent;
}
