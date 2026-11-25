package Tenancy { part def Mission; }
package MissionModel { part def Layer; item def MissionModelRevision; }
package Catalog {
    doc /* A small cross-domain navigation fixture. */
    part def CatalogDatabase {
        part mission : Tenancy::Mission;
        item revisions : CatalogRevision[*];
        attribute missing : MissingType;
    }
    item def CatalogRevision {
        part database : CatalogDatabase;
        part layer : MissionModel::Layer;
        item missionModelRevision : MissionModel::MissionModelRevision;
    }
}
