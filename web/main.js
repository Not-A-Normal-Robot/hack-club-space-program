/// <reference lib="dom" />
// @ts-check

/** @private @const */
const IDS = {
    LOADING_OVERLAY: "l",
    LOADING_TEXT: "t",
    LOADING_PROGRESS: "p",
    CANVAS: "h",
};

/**
 * @type {[
 *  HTMLDivElement | null,
 *  HTMLPreElement | null,
 *  HTMLProgressElement | null,
 *  HTMLCanvasElement | null,
 * ]}
 */
let [
    LOADING_OVERLAY,
    LOADING_TEXT,
    LOADING_PROGRESS,
    CANVAS
] = /** @type {*} */ ([
    IDS.LOADING_OVERLAY,
    IDS.LOADING_TEXT,
    IDS.LOADING_PROGRESS,
    IDS.CANVAS,
].map(id => document.getElementById(id)));

const MULTITHREADED = "SharedArrayBuffer" in globalThis;
const WASM_DIR = MULTITHREADED ? "./multithreaded" : "./singlethreaded";
const WASM_PATH = WASM_DIR + "/hack-club-space-program_bg.wasm";
const WBG = import(WASM_DIR + "/hack-club-space-program.js");

/**
 * @private
 * @returns {number}
 */
function getWasmMultithreadedBytes()
{
    return /*{{INLINER:WASM_MULTI_BYTES}}*/ 0;
}

/**
 * @private
 * @returns {number}
 */
function getWasmSinglethreadedBytes()
{
    return /*{{INLINER:WASM_SINGLE_BYTES}}*/ 0;
}

/**
 * @private
 * @returns {number}
 */
function getWasmTotalBytes()
{
    if (MULTITHREADED)
        return getWasmMultithreadedBytes();
    else
        return getWasmSinglethreadedBytes();
}

/**
 * @private
 * @param {string} stage
 * @param {string|Error} message
 */
function displayLoadError(stage, message)
{
    console.error(`Error during ${stage}`, message);
    if (LOADING_PROGRESS)
    {
        LOADING_PROGRESS.remove();
    }
    if (LOADING_TEXT)
    {
        /** @type {string} */
        let detail;
        if (typeof message === "string")
        {
            detail = message;
        } else
        {
            detail = message.message + "\n(see console for more)";
        }

        LOADING_TEXT.textContent = `Error occurred while ${stage}\n${detail}`;
    }
    if (CANVAS)
    {
        CANVAS.ariaBusy = "false";
    }
}

/**
 * @param {number} loaded The amount of bytes loaded.
 */
function displayDlProgress(loaded)
{
    const totalBytes = getWasmTotalBytes();
    if (LOADING_PROGRESS)
    {
        LOADING_PROGRESS.max = totalBytes;
        LOADING_PROGRESS.value = loaded;
    }
    if (LOADING_TEXT)
    {
        const loadedMB = (loaded / 1000000).toPrecision(3);
        const totalMB = (totalBytes / 1000000).toPrecision(3);
        const percentage = (loaded * 100 / totalBytes).toPrecision(3);
        LOADING_TEXT.textContent =
            `Downloading WASM file: ${loadedMB} / ${totalMB} MB (${percentage}%)`;
    }
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
        displayLoadError("initially fetching WASM", /** @type {Error} *//** @type {*} */(e));
        return;
    }

    if (!response.ok)
    {
        displayLoadError(
            "initially fetching WASM",
            `Response returned code ${response.status}, with text ${response.statusText}`
        );
        return;
    }

    const body = response.body;

    if (!body)
    {
        displayLoadError(
            "initially fetching WASM",
            "Response body was empty",
        );
        return;
    }

    const totalBytes = getWasmTotalBytes();
    const binary = new Uint8Array(totalBytes);
    let loadedBytes = 0;

    try
    {
        for await (const chunk of body)
        {
            const newLoadedBytes = loadedBytes + chunk.byteLength;

            if (newLoadedBytes > totalBytes)
            {
                displayLoadError(
                    "downloading WASM",
                    `Expected WASM to be ${totalBytes} bytes, but it was more`
                );
                return;
            }

            binary.set(chunk, loadedBytes);
            loadedBytes = newLoadedBytes;

            displayDlProgress(loadedBytes);
        }
    } catch (e)
    {
        displayLoadError("downloading WASM", /** @type {Error} *//** @type {*} */(e));
        return;
    }

    if (loadedBytes !== totalBytes)
    {
        displayLoadError(
            "Post-WASM download",
            `Expected WASM to be ${totalBytes} bytes, got ${loadedBytes} bytes`
        );
        return;
    }

    if (LOADING_PROGRESS)
    {
        LOADING_PROGRESS.value = 0;
        LOADING_PROGRESS.max = 0;
    }
    if (LOADING_TEXT)
    {
        LOADING_TEXT.textContent = "Compiling WASM file...";
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

    if (LOADING_TEXT)
    {
        LOADING_TEXT.textContent = "Fetching WBG script...";
    }

    /** @type {function({module_or_path: WebAssembly.Module}): *} */
    let wbg;
    try
    {
        wbg = (await WBG).default;
    } catch (e)
    {
        displayLoadError(
            "WBG load",
            /** @type {Error} *//** @type {*} */(e)
        );
        return;
    }

    if (LOADING_TEXT)
    {
        LOADING_TEXT.textContent = "Instantiating WASM module...";
    }

    await wbg({ module_or_path: module });

    if (LOADING_OVERLAY)
    {
        LOADING_OVERLAY.remove();
    }
    if (CANVAS)
    {
        CANVAS.ariaBusy = "false";
        CANVAS.ariaDescribedByElements = null;
    }
}

main();

addEventListener("DOMContentLoaded", () =>
{
    [LOADING_OVERLAY, LOADING_TEXT, LOADING_PROGRESS, CANVAS] = /** @type {*} */ ([
        IDS.LOADING_OVERLAY,
        IDS.LOADING_TEXT,
        IDS.LOADING_PROGRESS,
        IDS.CANVAS,
    ].map(id => document.getElementById(id)));
});
