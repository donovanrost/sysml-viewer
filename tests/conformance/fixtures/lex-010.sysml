// Quoted keywords, short names, aliases, case and dialect boundaries.
package KeywordsSysML {
    part def <'true'> 'part';
    alias <'false'> 'return' for 'part';
    part quotedType : 'part';
    part viaShort : 'true';
    part viaAlias : 'return';
    part viaAliasShort : 'false';
    part def feature;
    part foreign : feature;
    part def PART;
    part uppercase : PART;
    part def 'new';
    part quotedNew : 'new';
    attribute 'and' = 3;
    part bound['and'];
}
