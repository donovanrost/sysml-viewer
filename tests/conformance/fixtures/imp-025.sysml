package CollisionFirst { part def A; alias Shared for A; }
package CollisionSecond { part def B; alias Shared for B; }
package CollisionConsumer {
    private import CollisionFirst::*;
    private import CollisionSecond::*;
}
package CollisionOwned {
    part def Shared;
    private import CollisionFirst::*;
}
