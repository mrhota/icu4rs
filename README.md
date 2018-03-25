Internationalization Components for Unicode for Rust, or ICU4RS,
provides internationalization and localization support to applications
in the spirit of [ICU].

# Doneness

Not even close. It's just a glint in my eye, at the moment. I would love
help!

# Overview

A user would use this library to 1) get access to CLDR (and other)
internationalization data, and 2) use the library's API to perform
translations, format strings, numbers, dates, currencies, phone numbers,
etc. into a given locale's format and language.

To begin with, we'll just use ICU data, which is easily accessible in
source or binary format and contains nearly everything you could
possibly want in terms of charset conversions, locale, cultural, and
regional data, and more.

**Thus, we first want to get a set of ICU data decoders in place.**

Depending on needs and maintainability, we'll consider adding decoders
for other kinds of locale data (think GNU's C locale data).

# CLDR Data

In this library's current state, you'll need to get the ICU4J binary
data yourself. Visit ICU's [download page], look under the ICU4J
section, and download the file you want according to the following two
options:

  1. The `tgz` file described as something like `gzipped tar archive
  including the entire source package`. For ICU4J 60.2, it's called
  `icu4j-60_2.tgz`.
      - It contains two jar files---`icudata.jar` and
      `icutzdata.jar`---containing the binary data files usable by this
      library. Once you have those jars, you can just `unzip` them into
      this project's data directory (location TBD).
  1. The `jar` file described as something like "core binaries" jar
  file.
      - All data lives under the `com/ibm/icu/impl/data/icudt60b/` which
      you should put into ICU4RS's data directory (location TDB).

You can find a ton of additional information about ICU data here:
http://userguide.icu-project.org/icudata, including information about
how to build the data yourself from the sources in the ICU4C repo.

# License

Dual-licensed under Unlicense or MIT unless otherwise noted.

[ICU]: https://www.icu-project.org
[download page]: http://site.icu-project.org/download/60
