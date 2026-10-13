package Left { part def Duplicate; }
package Right { part def Duplicate; }
package Problems {
    private import Left::*;
    private import Right::*;
    part ambiguous : Duplicate;
    part missing : Absent;
}
