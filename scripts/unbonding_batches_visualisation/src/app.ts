import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import {
  Client as DropCoreClient,
  UnbondBatch,
} from "../../../integration_tests/src/generated/contractLib/dropCore";

/* There're 2 possible modes for script
 * RECENT will retrieve all recent batches since it'll meet one 'withdrawn'
 * FULL will retrieve all batches from current batch to 0-nth
 */
enum Mode {
  RECENT,
  FULL,
}

const MODE: string = process.env.MODE;
const CORE_CONTRACT: string = process.env.CORE_CONTRACT;
const NODE_ADDRESS: string = process.env.NODE_ADDRESS;
const WALLET_MNEMONIC: string = process.env.WALLET_MNEMONIC;

/* addLeadingZeros used there to add leading zeros to date
 * We need date formatting for pretty output
 */
function addLeadingZeros(num: number, targetLength: number): string {
  let numStr: string = num.toString();
  while (numStr.length < targetLength) {
    numStr = "0" + numStr;
  }

  return numStr;
}

/*
 * batch_id - number of batch in contract's order
 * status - batch.status
 * expected_amount - batch.expected_amount
 * creation_time - batch.creation_time
 * expected_finalization_time - (batch.creation_time + core.config.unbonding_period / 7)
 * unstaked_amount - batch.unbonded_amount
 */
type BatchInfo = {
  batch_id: number;
  status: string;
  expected_amount: number;
  creation_time: string;
  expected_finalization_time: string;
  unstaked_amount: number;
};

/* Function print_n serves for getting information about 'n' first batches
 * current_unbond_batch - latest unbonding batch gotten from query
 * n - first n batches starting from current_unbond_batch
 * drop_client - drop client generated code from binary, used for queries
 */
async function print_n(
  current_unbond_batch: number,
  n: number,
  drop_client: DropCoreClient
): Promise<Array<BatchInfo>> {
  if (current_unbond_batch - n < 0) {
    return [];
  }
  const dropCoreConfig = await drop_client.queryConfig();
  let arr = [];

  for (; n >= 0; n -= 1, current_unbond_batch -= 1) {
    let batch: UnbondBatch = await drop_client.queryUnbondBatch({
      batch_id: String(current_unbond_batch),
    });

    let creation_time: any = new Date(batch.status_timestamps.new * 1000);
    creation_time = {
      day: addLeadingZeros(creation_time.getUTCDate(), 2),
      month: addLeadingZeros(creation_time.getUTCMonth(), 2),
      year: creation_time.getUTCFullYear(),
      hours: addLeadingZeros(creation_time.getUTCHours(), 2),
      minutes: addLeadingZeros(creation_time.getUTCMinutes(), 2),
      seconds: addLeadingZeros(creation_time.getUTCSeconds(), 2),
    };

    if (batch.status !== "new") {
      let expected_finalization_time: any = new Date(
        1000 *
          (batch.status_timestamps.unbond_requested +
            dropCoreConfig.unbonding_period +
            dropCoreConfig.unbond_batch_switch_time)
      );
      expected_finalization_time = {
        day: addLeadingZeros(expected_finalization_time.getUTCDate(), 2),
        month: addLeadingZeros(expected_finalization_time.getUTCMonth(), 2),
        year: expected_finalization_time.getUTCFullYear(),
        hours: addLeadingZeros(expected_finalization_time.getUTCHours(), 2),
        minutes: addLeadingZeros(expected_finalization_time.getUTCMinutes(), 2),
        seconds: addLeadingZeros(expected_finalization_time.getUTCSeconds(), 2),
      };

      arr.push({
        batch_id: current_unbond_batch,
        status: batch.status,
        expected_amount: batch.expected_amount,
        creation_time: `${creation_time.day}/${creation_time.month}/${creation_time.year}(${creation_time.hours}:${creation_time.minutes}:${creation_time.seconds})`,
        expected_finalization_time: `${expected_finalization_time.day}/${expected_finalization_time.month}/${expected_finalization_time.year}(${expected_finalization_time.hours}:${expected_finalization_time.minutes}:${expected_finalization_time.seconds})`,
        unstaked_amount: batch.unbonded_amount,
      });
    } else {
      arr.push({
        batch_id: current_unbond_batch,
        status: batch.status,
        expected_amount: batch.expected_amount,
        creation_time: `${creation_time.day}/${creation_time.month}/${creation_time.year}(${creation_time.hours}:${creation_time.minutes}:${creation_time.seconds})`,
        expected_finalization_time: null,
        unstaked_amount: batch.unbonded_amount,
      });
    }
  }

  return arr;
}

async function main(mode: Mode): Promise<void> {
  const mainWallet: DirectSecp256k1HdWallet =
    await DirectSecp256k1HdWallet.fromMnemonic(WALLET_MNEMONIC, {
      prefix: "neutron",
    });
  const clientCW: SigningCosmWasmClient =
    await SigningCosmWasmClient.connectWithSigner(NODE_ADDRESS, mainWallet, {
      gasPrice: GasPrice.fromString("0.75untrn"),
    });
  const drop_client: DropCoreClient = new DropCoreClient(
    clientCW,
    CORE_CONTRACT
  );
  let res: Array<BatchInfo> = [];

  switch (mode) {
    case Mode.RECENT: {
      let unbond_batch_height: number = Number(
        await drop_client.queryCurrentUnbondBatch()
      );
      let current_unbond_batch: number = unbond_batch_height;
      let batch: UnbondBatch = await drop_client.queryUnbondBatch({
        batch_id: String(current_unbond_batch),
      });
      let n = 0;

      /* Get amount of batches that haven't withdrawn yet
       * Provide given n as n-1 since count there starts with 0
       */
      while (current_unbond_batch > 0 && batch.status !== "withdrawn") {
        current_unbond_batch -= 1;
        batch = await drop_client.queryUnbondBatch({
          batch_id: String(current_unbond_batch),
        });
        n += 1;
      }
      res = await print_n(unbond_batch_height, n == 0 ? 0 : n - 1, drop_client);
      break;
    }
    case Mode.FULL: {
      let current_unbond_batch: number = Number(
        await drop_client.queryCurrentUnbondBatch()
      );
      res = await print_n(
        current_unbond_batch,
        current_unbond_batch,
        drop_client
      );
      break;
    }
  }
  console.log(JSON.stringify(res));
}

let mode: Mode;

switch (MODE) {
  case "RECENT": {
    mode = Mode.RECENT;
    break;
  }
  case "FULL": {
    mode = Mode.FULL;
    break;
  }
  default: {
    throw new Error(`Unknown mode given: ${MODE}`);
  }
}

main(mode);
