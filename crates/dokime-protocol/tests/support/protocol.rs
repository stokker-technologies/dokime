/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Regenerate Wire Bindings With the Pinned Protobuf Compiler.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    fs,
    path::{Path, PathBuf},
};

// Third Party Imports
use heck::{ToSnakeCase, ToUpperCamelCase};
use prost::Message;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn compiler(root: &Path) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    // Archived Tests Consume the Transferred Compiler, Not the Producer's Registry.
    let (compiler, includes) = if let Ok(profile) = std::env::var("TEST_PROFILE") {
        let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let vendor = root.join("target/build").join(profile).join(target).join("fixtures/inputs/vendor");
        let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => "linux-x86_64",
            ("linux", "aarch64") => "linux-aarch_64",
            ("macos", "x86_64") => "macos-x86_64",
            ("macos", "aarch64") => "macos-aarch_64",
            ("windows", _) => "win32",
            other => panic!("no vendored protoc for {other:?}"),
        };
        let prefix = format!("protoc-bin-vendored-{platform}-");
        let package = fs::read_dir(vendor)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .find(|path| path.file_name().unwrap().to_string_lossy().starts_with(&prefix))
            .expect("the transferred vendor inputs contain the pinned protobuf compiler");
        let executable = if cfg!(windows) { "protoc.exe" } else { "protoc" };
        (package.join("bin").join(executable), package.join("include"))
    } else {
        (protoc_bin_vendored::protoc_bin_path()?, protoc_bin_vendored::include_path()?)
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&compiler)?.permissions();
        permissions.set_mode(permissions.mode() | 0o100);
        fs::set_permissions(&compiler, permissions)?;
    }

    Ok((compiler, includes))
}

pub fn generate(root: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let proto = root.join("proto");
    let (compiler, includes) = self::compiler(root)?;
    let mut inputs = fs::read_dir(proto.join("dokime/v1"))?.map(|entry| entry.map(|entry| entry.path())).collect::<Result<Vec<_>, _>>()?;
    inputs.retain(|path| path.extension().is_some_and(|extension| extension == "proto"));
    inputs.sort();
    let mut config = prost_build::Config::new();
    config.protoc_executable(&compiler);
    config.extern_path(".harmos.v1", "crate::harmos::v1");
    config.file_descriptor_set_path(output.join("descriptors.bin"));
    tonic_prost_build::configure().build_client(true).build_server(true).out_dir(output).compile_with_config(
        config,
        &inputs,
        &[proto.clone(), includes.clone()],
    )?;
    let mut descriptors = prost_types::FileDescriptorSet::decode(fs::read(output.join("descriptors.bin"))?.as_slice())?;
    for file in &mut descriptors.file {
        file.source_code_info = None;
    }
    let descriptors = descriptors.encode_to_vec();
    fs::write(output.join("descriptors.bin"), &descriptors)?;
    let pool = prost_reflect::DescriptorPool::decode(descriptors.as_slice())?;
    let mut reflection = String::from("// @generated from authoritative protobuf descriptors.\n");
    for message in pool.all_messages().filter(|message| message.package_name() == "dokime.v1" && !message.is_map_entry()) {
        let mut parents = Vec::new();
        let mut parent = message.parent_message();
        while let Some(message) = parent {
            parents.push(message.name().to_snake_case());
            parent = message.parent_message();
        }
        parents.reverse();
        parents.push(message.name().to_upper_camel_case());
        let path = parents.join("::");
        reflection.push_str(&format!(
            "impl prost_reflect::ReflectMessage for crate::v1::{path} {{\n\
             fn descriptor(&self) -> prost_reflect::MessageDescriptor {{\n\
             crate::descriptor_pool().get_message_by_name(\"{}\").expect(\"generated protocol descriptor\")\n\
             }}\n}}\n",
            message.full_name(),
        ));
    }
    fs::write(output.join("reflection.rs"), reflection)?;
    let mut config = prost_build::Config::new();
    config.protoc_executable(compiler);
    tonic_prost_build::configure().build_client(true).build_server(true).out_dir(output).compile_with_config(
        config,
        &[proto.join("harmos/v1/plugin.proto")],
        &[proto, includes],
    )?;
    let python = output.join("python");
    let csharp = output.join("csharp");
    let cpp = output.join("cpp");
    for path in [&python, &csharp, &cpp] {
        fs::create_dir_all(path)?;
    }
    let (compiler, includes) = self::compiler(root)?;
    let proto = root.join("proto");
    let document_messages = output.join("document-messages");
    fs::create_dir_all(&document_messages)?;
    let mut config = prost_build::Config::new();
    config.protoc_executable(compiler);
    config.out_dir(document_messages);
    config.file_descriptor_set_path(python.join("documents.pb"));
    config.skip_source_info();
    config.protoc_arg(format!("--csharp_out={}", csharp.display()));
    config.protoc_arg(format!("--cpp_out={}", cpp.display()));
    config.compile_protos(&[proto.join("dokime/v1/values.proto"), proto.join("dokime/v1/documents.proto")], &[proto, includes])?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
