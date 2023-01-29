// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use ocamlrep_ocamlpool::ocaml_ffi;

extern "C" {
    fn ocamlpool_enter();
    fn ocamlpool_reserve_block(tag: u8, size: usize) -> usize;
    fn ocamlpool_leave();
}

// This test attempts to catch off by one issues in ocamlpool.c

// Magic constant needs to fulfill two requirements:
// Needs to be above the OCAMLPOOL_DEFAULT_SIZE constant in ocamlpool.h
//   This requirement is easy to fulfill
// Needs to be the exact size of memory block allocated by ocamlpool_reserve_block
//   which is given by the Chunk_size call in chunk_alloc in ocamlpool.c
//   This requirement requires some magic
const MAGIC_MEMORY_SIZE: usize = 1053183;

ocaml_ffi! {
    fn test() {
        unsafe {
            ocamlpool_enter();
            // This line will crash on off by one error
            ocamlpool_reserve_block(0, MAGIC_MEMORY_SIZE);
            ocamlpool_leave();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn workspace_dir() -> std::path::PathBuf {
        let output = std::process::Command::new("cargo")
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;
        let cargo_path = std::path::Path::new(std::str::from_utf8(&output).unwrap().trim());
        cargo_path.parent().unwrap().to_path_buf()
    }

    #[test]
    fn ocamlpool_test() {
        let mut targets: std::path::PathBuf = workspace_dir();
        targets.push("target");
        targets.push("debug");

        let mut compile = Command::new("ocamlopt.opt");
        compile.args([
            "-verbose",
            "-c",
            "ocamlpool_test.ml",
            "-o",
            "ocamlpool_test_ml.cmx",
        ]);
        let mut link = Command::new("ocamlopt.opt");
        let link_search_path_flag = "-L".to_owned() + targets.as_path().to_str().unwrap();
        link.args([
            "-verbose",
            "-o",
            "ocamlpool_test",
            "ocamlpool_test_ml.cmx",
            "-ccopt",
            link_search_path_flag.as_str(),
            "-cclib",
            "-locamlpool_test",
            "-cclib",
            "-locamlpool",
        ]);
        let mut p = compile.spawn().unwrap();
        p.wait().ok().unwrap();
        let mut p = link.spawn().unwrap();
        p.wait().ok().unwrap();

        let mut ocamlpool_test = Command::new("sh");
        ocamlpool_test.args(["-c", "./ocamlpool_test"]);
        let mut p = ocamlpool_test.spawn().unwrap();
        p.wait().ok().unwrap();
    }
}
