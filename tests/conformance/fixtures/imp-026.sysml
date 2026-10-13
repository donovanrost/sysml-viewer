package MissingImport { private import MissingProvider::*; }
package IncompleteSource { part def Base :> MissingBase; }
package IncompleteImport { private import IncompleteSource::Base::*; }
package FilterSource { part def A; }
package FilterImport { private import FilterSource::*[false]; }
package FilterFalse { private import FilterSource::*; filter false; }
