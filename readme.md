# Nadi Plugin for HEC-DSS Files

Work in Progress

This plugin can also be used as an example to see how a C library can be used to make a nadi plugin in rust.

# Features

- Read the catalogue of a DSS v7 file and list the timeseries paths.

# Future Plans
- Read timeseries values,
- Read timeseries with gaps,
- Write timeseries,
- extract/save other dss compatible data,
- insert network information in DSS file so nadi can directly load it.


# Note

If you get undefined symbol in the `libnadi_dss.so` caused due to `heclib.a` not having that symbol defined, then you might have to fix it from `hecdss-sys` with additional line like this:

	println!("cargo:rustc-link-lib=heclib");``
