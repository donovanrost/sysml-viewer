package ScopeFixture {
    part def Device;
    package Inner {
        part def Device;
        part local : Device;
        part global : $::ScopeFixture::Device;
        package Deep { part enclosing : Device; }
    }
}
