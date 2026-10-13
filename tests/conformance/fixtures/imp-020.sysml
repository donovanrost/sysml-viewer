package ImportSource {
    part def Engine;
    alias <E> Alias for Engine;
    private part def Hidden;
    protected part def Guarded;
}
package ImportConsumer {
    protected import ImportSource::Alias;
    private import ImportSource::*;
}
package ImportReexport {
    public import ImportSource::*;
}
package ImportDownstream {
    private import ImportReexport::*;
}
package ImportEmpty;
package ImportLibrary { public import ScalarValues::Integer; }
