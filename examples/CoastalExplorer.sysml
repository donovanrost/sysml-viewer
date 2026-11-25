package CoastalExplorer {
    doc /* An autonomous vessel for coastal research.
       Open SurveyVessel to explore its onboard systems and connections. */

    attribute def Percentage;
    attribute def Metres;

    port def PowerPort;
    port def DataPort;

    part def Battery {
        doc /* Rechargeable energy storage for a full survey mission. */
        attribute charge : Percentage = 100;
        port supply : PowerPort;
    }

    part def Propulsion {
        doc /* Electric propulsion with a command interface. */
        port power : ~PowerPort;
        port command : DataPort;
    }

    part def Navigation {
        doc /* Navigation controller that plans and tracks the survey route. */
        port power : ~PowerPort;
        port command : ~DataPort;
        port measurements : DataPort;
    }

    part def Sonar {
        doc /* Maps the seafloor and reports measurements to navigation. */
        attribute range : Metres = 80;
        port power : ~PowerPort;
        port measurements : ~DataPort;
    }

    requirement def SurveyEndurance {
        doc /* The vessel shall complete a coastal survey without recharging. */
    }

    part def ResearchVessel {
        doc /* Shared vessel systems. Specialized vessels inherit these parts
           and their connections. */
        part battery : Battery;
        part propulsion : Propulsion;
        part navigation : Navigation;
        connect battery.supply to propulsion.power;
        connect battery.supply to navigation.power;
        connect navigation.command to propulsion.command;
    }

    part def SurveyVessel :> ResearchVessel {
        doc /* A research vessel specialized for seafloor surveys. Adds sonar
           to the inherited battery, propulsion and navigation systems. */
        part sonar : Sonar;
        connect battery.supply to sonar.power;
        connect sonar.measurements to navigation.measurements;
    }

    part researchVessel : SurveyVessel;
}
