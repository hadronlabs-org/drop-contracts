import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import {
  Client as DropCoreClient,
  UnbondBatch,
} from "../../../integration_tests/src/generated/contractLib/dropCore";

const CORE_CONTRACT: string = process.env.CORE_CONTRACT;
const NODE_ADDRESS: string = process.env.NODE_ADDRESS;
const WALLET_MNEMONIC: string = process.env.WALLET_MNEMONIC;

async function main(): Promise<void> {
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

  const unbonding_period: number = (await drop_client.queryConfig())
    .unbonding_period;

  let current_unbond_batch: string =
    await drop_client.queryCurrentUnbondBatch();

  let batch: UnbondBatch = await drop_client.queryUnbondBatch({
    batch_id: current_unbond_batch,
  });

  while (batch.status !== "withdrawn") {
    console.log({
      batch_id: current_unbond_batch,
      status: batch.status,
      expected_amount: batch.expected_amount,
      creation_time: batch.created,
      expected_finalization_time: Math.floor(
        batch.created + unbonding_period / 7
      ),
      unstaked_amount: batch.unbonded_amount,
    });
    current_unbond_batch = String(Number(current_unbond_batch) - 1);
    batch = await drop_client.queryUnbondBatch({
      batch_id: current_unbond_batch,
    });
  }
}

main();
