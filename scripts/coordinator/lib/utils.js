"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runQueryRelayer = void 0;
const child_process_1 = require("child_process");
function runQueryRelayer(context, log, queryIds) {
    try {
        const stdout = (0, child_process_1.execSync)(`${context.config.coordinator.icqRunCmd} -q ${queryIds.join(' -q ')}`);
        log.debug(`stdout: ${stdout}`);
    }
    catch (error) {
        log.error(`Error running query relayer: ${error.message}`);
    }
}
exports.runQueryRelayer = runQueryRelayer;
