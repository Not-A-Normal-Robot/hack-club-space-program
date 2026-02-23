// @ts-check
/** @suppress {moduleLoad} */
const WBG = import("./hack-club-space-program.js");
const WASM_PATH = "./hack-club-space-program_bg.wasm";

/**
 * @private
 * @returns {number}
 */
function getWasmTotalBytes()
{
    return /*{{INLINER:WASM_TOTAL_SIZE}}*/ 0;
}

/**
 * @private
 * @param {string} stage
 * @param {string|Error} message
 */
function displayLoadError(stage, message)
{
    console.error(`Error during ${stage}`, message);
    // TODO: Error Screen
}

/**
 * @param {number} loaded The amount of bytes loaded.
 */
function displayDlProgress(loaded)
{
    console.log(`Loaded ${loaded}/${getWasmTotalBytes()}`);
    // TODO: Progress Screen
}

async function main()
{
    /** @type {Response} */
    let response;
    try
    {
        response = await fetch(WASM_PATH);
    } catch (e)
    {
        displayLoadError("initial WASM fetch", /** @type {Error} *//** @type {*} */(e));
        return;
    }

    if (!response.ok)
    {
        displayLoadError(
            "initial WASM fetch",
            `Response returned code ${response.status}, with text ${response.statusText}`
        );
        return;
    }

    const body = response.body;

    if (!body)
    {
        displayLoadError(
            "initial WASM fetch",
            "Response body was empty",
        );
        return;
    }

    const binary = new Uint8Array(getWasmTotalBytes());
    let loadedBytes = 0;

    try
    {
        for await (const chunk of body)
        {
            const newLoadedBytes = loadedBytes + chunk.byteLength;

            if (newLoadedBytes > getWasmTotalBytes())
            {
                displayLoadError(
                    "WASM downloading",
                    `Expected WASM to be ${getWasmTotalBytes()} bytes, but it was more`
                );
                return;
            }

            binary.set(chunk, loadedBytes);
            loadedBytes = newLoadedBytes;

            displayDlProgress(loadedBytes);
        }
    } catch (e)
    {
        displayLoadError("WASM downloading", /** @type {Error} *//** @type {*} */(e));
        return;
    }

    if (loadedBytes !== getWasmTotalBytes())
    {
        displayLoadError(
            "Post-WASM download",
            `Expected WASM to be ${getWasmTotalBytes()} bytes, got ${loadedBytes} bytes`
        );
        return;
    }

    /** @type {WebAssembly.Module} */
    let module;
    try
    {
        module = await WebAssembly.compile(binary);
    } catch (e)
    {
        displayLoadError("WASM compilation", /** @type {Error} *//** @type {*} */(e));
        return;
    }

    /** @type {function({module_or_path: WebAssembly.Module}): *} */
    let init;
    try
    {
        init = (await WBG).default;
    } catch (e)
    {
        displayLoadError(
            "WBG load",
            /** @type {Error} *//** @type {*} */(e)
        );
        return;
    }

    init({ module_or_path: module });
}

main();

