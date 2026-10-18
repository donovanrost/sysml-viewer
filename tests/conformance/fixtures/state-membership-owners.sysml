package StateMembershipOwners {
    part def WrongPart {
        entry action entering;
        do action running;
        exit;
    }
    action def WrongAction { entry; }
    state def ValidDefinition { entry; do; exit; }
    state validUsage { entry; do; exit; }
    exhibit state validExhibit { entry; do; exit; }
}
