import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import {
  Client as DropCoreClient,
  Config as DropCoreConfig,
  UnbondBatch,
} from "drop-ts-client/src/contractLib/dropCore";

const MODE: string = process.env.MODE;
const CORE_CONTRACT: string = process.env.CORE_CONTRACT;
const NODE_ADDRESS: string = process.env.NODE_ADDRESS;
const WALLET_MNEMONIC: string = process.env.WALLET_MNEMONIC;

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
  expected_amount: string;
  creation_time: string;
  expected_finalization_time: string;
  unstaked_amount: string;
};

type CoreBatchInformaion = {
  batch_id: number;
  details: UnbondBatch;
};

/* Function create_batch_info serves for collecting
 * all important information about batch into 1 single structure
 */
async function create_batch_info(
  dropCoreConfig: DropCoreConfig,
  batch: CoreBatchInformaion
): Promise<BatchInfo> {
  const creation_date = new Date(batch.details.status_timestamps.new * 1000);
  const creation_time = {
    day: creation_date.getUTCDate().toString().padStart(2, "0"),
    month: creation_date.getUTCMonth().toString().padStart(2, "0"),
    year: creation_date.getUTCFullYear(),
    hours: creation_date.getUTCHours().toString().padStart(2, "0"),
    minutes: creation_date.getUTCMinutes().toString().padStart(2, "0"),
    seconds: creation_date.getUTCSeconds().toString().padStart(2, "0"),
  };

  let batch_details: BatchInfo = {
    batch_id: batch.batch_id,
    status: batch.details.status,
    expected_amount: batch.details.expected_native_asset_amount,
    creation_time: `${creation_time.day}/${creation_time.month}/${creation_time.year}(${creation_time.hours}:${creation_time.minutes}:${creation_time.seconds})`,
    expected_finalization_time: null,
    unstaked_amount: batch.details.unbonded_amount,
  };
  if (batch.details.status !== "new") {
    const expected_finalization_date = new Date(
      1000 *
        (batch.details.status_timestamps.unbond_requested +
          dropCoreConfig.unbonding_period +
          dropCoreConfig.unbond_batch_switch_time)
    );
    const expected_finalization_time = {
      day: expected_finalization_date.getUTCDate().toString().padStart(2, "0"),
      month: expected_finalization_date
        .getUTCMonth()
        .toString()
        .padStart(2, "0"),
      year: expected_finalization_date.getUTCFullYear(),
      hours: expected_finalization_date
        .getUTCHours()
        .toString()
        .padStart(2, "0"),
      minutes: expected_finalization_date
        .getUTCMinutes()
        .toString()
        .padStart(2, "0"),
      seconds: expected_finalization_date
        .getUTCSeconds()
        .toString()
        .padStart(2, "0"),
    };
    batch_details.expected_finalization_time = `${expected_finalization_time.day}/${expected_finalization_time.month}/${expected_finalization_time.year}(${expected_finalization_time.hours}:${expected_finalization_time.minutes}:${expected_finalization_time.seconds})`;
  }

  return batch_details;
}

async function handle_batches(
  drop_client: DropCoreClient,
  callback?: (batch: UnbondBatch) => boolean
): Promise<Array<BatchInfo>> {
  const res: Array<BatchInfo> = [];
  const config = await drop_client.queryConfig();
  for (
    let current_batch = Number(await drop_client.queryCurrentUnbondBatch());
    current_batch >= 0;
    current_batch -= 1
  ) {
    const batch = await drop_client.queryUnbondBatch({
      batch_id: current_batch.toString(),
    });
    if (callback && callback(batch)) {
      break;
    }
    res.push(
      await create_batch_info(config, {
        batch_id: current_batch,
        details: batch,
      })
    );
  }
  return res;
}

/* There're 2 possible modes for script
 * RECENT will retrieve all recent batches since it'll meet one 'withdrawn'
 * FULL will retrieve all batches from current batch to 0-nth
 */
async function main(mode: string): Promise<void> {
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
    case "RECENT": {
      res = await handle_batches(
        drop_client,
        (batch) => batch.status === "withdrawn"
      );
      break;
    }
    case "FULL": {
      res = await handle_batches(drop_client);
      break;
    }
    default: {
      throw new Error(`Invalid mode given: ${mode}`);
    }
  }
  console.log(JSON.stringify(res));
}

main(MODE);
