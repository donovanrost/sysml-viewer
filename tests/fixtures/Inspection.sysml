package Inspection {
    doc /* A file-loading fixture with a cross-file type and a preview limitation. */
    part 'Pump <A>' : Library::Motor;
    part receiver;
    connect 'Pump <A>'.output to receiver;
    import External::*;
}
