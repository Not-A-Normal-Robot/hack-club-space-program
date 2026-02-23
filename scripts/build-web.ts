#!/usr/bin/env -S deno -P
"use strict";

if (!("Deno" in globalThis))
{
    throw new Error("Deno object not found; please run this script using Deno. You can get deno at https://deno.land/");
}

import * as path from "@std/path";
import * as fs from "@std/fs";

const DIRNAME = import.meta.dirname;
if (!DIRNAME)
{
    throw new Error("`import.meta.dirname` is undefined. This script must be run locally.");
}

const GAME_NAME = "hack-club-space-program";
const OUT_DIR = path.join(DIRNAME, "../target/wasm32-unknown-unknown/web");
const RELEASE_MODE = Deno.args.includes("--release");
const WASM_PATH = path.join(
    DIRNAME,
    "../target/wasm32-unknown-unknown",
    RELEASE_MODE ? "release" : "debug",
    GAME_NAME + ".wasm",
);
const WEB_ADDITIONS_PATH = path.join(DIRNAME, "../web");
const ASSETS_PATH = path.join(DIRNAME, "../assets");

/** Checks if the given executable is accessible, by running `<cmd> --version`. */
async function isExecAvailable(cmd: string): Promise<boolean>
{
    try
    {
        const command = new Deno.Command(cmd, {
            args: ["--version"],
            stdout: "null",
            stdin: "null",
        });
        return (await command.output()).success;
    } catch (e)
    {
        if (e instanceof Deno.errors.NotFound)
        {
            return false;
        }
        throw e;
    }
}

/**
 * Generates an error for when an executed process fails
 */
function execError(
    action: string,
    cmd: string,
    code: number,
    stdout: Uint8Array,
    stderr: Uint8Array,
): Error
{
    return new Error(
        `An error occurred while ${action}:\n` +
        `'${cmd}' returned with exit code ${code}\n\n` +
        `===== ${cmd} stdout =====\n\n${stdout}` +
        `===== ${cmd} stderr =====\n\n${stderr}`
    );
}

async function checkDependencies()
{
    const results = await Promise.allSettled(["cargo", "rustup"].map((cmd) =>
    {
        return isExecAvailable(cmd)
            .then((available) =>
            {
                if (available)
                {
                    return Promise.resolve();
                } else
                {
                    return Promise.reject(`Could not access \`${cmd}\`: is it in your PATH?\n` +
                        "One way to get cargo, rustup, and the rest of the rust toolchain is rustup: https://rustup.rs");
                }
            });
    }));

    const rejections = results.filter(r => r.status === "rejected").map(r => r.reason);

    if (rejections.length > 0)
    {
        for (const rejection of rejections)
        {
            console.error(rejection);
        }
        Deno.exit(1);
    }
}

async function prepWasmBindgen()
{
    if (!await isExecAvailable("wasm-bindgen"))
    {
        console.log("wasm-bindgen not found, installing...");
        const command = new Deno.Command("cargo", {
            args: ["install", "wasm-bindgen-cli"],
            stdout: "piped",
        });
        const output = await command.output();
        if (output.success)
        {
            console.log("Finished installing wasm-bindgen");
        } else
        {
            throw new Error(
                "Error occurred installing wasm-bindgen\n\n" +
                `===== cargo stdout =====\n${output.stdout}\n\n` +
                `===== cargo stderr =====\n${output.stderr}`
            );
        }
    }
    if (!await isExecAvailable("wasm-bindgen"))
    {
        throw new Error("Could not access wasm-bindgen after installation, something must have gone awry");
    }
}

async function prepRustupTarget()
{
    console.log("Checking list of installed rustup targets");
    const listCommand = new Deno.Command("rustup", {
        args: ["target", "list", "--installed"],
        stdin: "null",
        stdout: "piped"
    });
    const listOutput = await listCommand.output();
    if (!listOutput.success)
    {
        throw execError(
            "checking rustup's installed targets list",
            "rustup",
            listOutput.code,
            listOutput.stdout,
            listOutput.stderr,
        );
    }

    const list = new TextDecoder().decode(listOutput.stdout).split("\n").map(s => s.trim());

    if (list.includes("wasm32-unknown-unknown"))
    {
        console.log("No need to install wasm32-unknown-unknown as it's already installed");
        return;
    }

    console.log("rustup wasm32-unknown-unknown target is not installed, installing...");

    const installCommand = new Deno.Command("rustup", {
        args: ["target", "install", "wasm32-unknown-unknown"],
        stdin: "null",
        stdout: "piped",
    });
    const installOutput = await installCommand.output();

    if (!installOutput.success)
    {
        throw execError(
            "installing the wasm32-unknown-unknown rustup target",
            "rustup",
            installOutput.code,
            installOutput.stdout,
            installOutput.stderr,
        );
    }

    console.log("Installed wasm32-unknown-unknown rustup target");
}

async function buildWasm()
{
    const args = ["build", "--target", "wasm32-unknown-unknown"];

    if (RELEASE_MODE)
    {
        args.push("--release");
    }

    console.log("Compiling the Rust codebase to WASM...");

    const command = new Deno.Command("cargo", {
        args,
        stdin: "null",
        stdout: "piped",
    });

    const output = await command.output();

    if (!output.success)
    {
        throw execError(
            "compiling Rust to WASM",
            "cargo",
            output.code,
            output.stdout,
            output.stderr,
        );
    }

    console.log("Rust to WASM compilation complete!");
}

async function bindWasm()
{
    console.log("Binding WASM to JS using wasm-bindgen...");
    const command = new Deno.Command("wasm-bindgen", {
        args: ["--target", "web", "--out-dir", OUT_DIR, "--out-name", GAME_NAME, WASM_PATH],
        stdin: "null",
        stdout: "piped",
    });

    const output = await command.output();

    if (!output.success)
    {
        throw execError(
            "running wasm-bindgen",
            "wasm-bindgen",
            output.code,
            output.stdout,
            output.stderr,
        );
    }
}

async function copyAssets(): Promise<void>
{
    if (await fs.exists(ASSETS_PATH))
    {
        await fs.copy(ASSETS_PATH, OUT_DIR, {
            overwrite: true,
        });
    }
}

async function copyWebAdditions(): Promise<void>
{
    await fs.copy(WEB_ADDITIONS_PATH, OUT_DIR, { overwrite: true });
}

async function copyAdditionalFiles()
{
    await Promise.all([copyAssets(), copyWebAdditions()]);
}

async function main()
{
    // Prep and checks
    await checkDependencies();
    await Promise.all([prepWasmBindgen, prepRustupTarget]);

    console.log("Clearing and recreating output directory...");
    await fs.ensureDir(OUT_DIR);
    await fs.emptyDir(OUT_DIR);

    await buildWasm();
    await bindWasm();
    // TODO: wasm-opt


    console.log("Copying additional files...");
    await copyAdditionalFiles();
}

await main();
console.log("Done!");