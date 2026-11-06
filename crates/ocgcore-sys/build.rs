use std::path::PathBuf;

const LUA_SOURCES: [&str; 26] = [
    "lapi", "lauxlib", "lbaselib", "lcode", "lctype", "ldebug", "ldo", "ldump", "lfunc", "lgc", "liolib", "llex", "lmathlib", "lmem",
    "lobject", "lopcodes", "lparser", "lstate", "lstring", "lstrlib", "ltable", "ltablib", "ltm", "lundump", "lvm", "lzio",
];

fn main() -> std::io::Result<()> {
    let core = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/ygopro-core");
    let lua = core.join("lua");
    println!("cargo:rerun-if-changed={}", core.display());

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .warnings(false)
        .include(lua.join("src"))
        .define("NOMINMAX", None);
    // Lua must be compiled as C++ so its errors unwind through ocgcore frames; the core rejects the duel otherwise.
    let customize = lua.join("luaconf-customize.h");
    if build.get_compiler().is_like_msvc() {
        build.flag("/TP").flag("/EHsc").flag(format!("/FI{}", customize.display()));
    } else {
        build.flag("-xc++").flag("-include").flag(customize.display().to_string());
    }
    build.files(LUA_SOURCES.iter().map(|name| lua.join("src").join(format!("{name}.c"))));
    for entry in std::fs::read_dir(&core)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "cpp") {
            build.file(path);
        }
    }
    build.compile("ocgcore");
    Ok(())
}
