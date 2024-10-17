import { waitFor } from './waitFor';
import { DropCore, DropPuppeteer, DropPuppeteerInitia } from 'drop-ts-client';
import { ResponseHookSuccessMsg } from 'drop-ts-client/lib/contractLib/dropCore';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { SigningStargateClient } from '@cosmjs/stargate';

const DropCoreClass = DropCore.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
const DropPuppeteerInitiaClass = DropPuppeteerInitia.Client;

export const waitForPuppeteerICQ = async (
  client: SigningStargateClient | SigningCosmWasmClient,
  coreContractClient?: InstanceType<typeof DropCoreClass>,
  puppeteerContractClient?:
    | InstanceType<typeof DropPuppeteerClass>
    | InstanceType<typeof DropPuppeteerInitiaClass>,
): Promise<void> => {
  const puppeteerResponse = (
    await coreContractClient.queryLastPuppeteerResponse()
  ).response as {
    success: ResponseHookSuccessMsg;
  };

  const block = await client.getBlock();

  let controlHeight = block.header.height;

  if (puppeteerResponse && puppeteerResponse.success) {
    controlHeight = puppeteerResponse.success.remote_height;
  }

  controlHeight++;

  const waitForBalances = waitFor(async () => {
    const { remote_height: remoteHeight } =
      (await puppeteerContractClient.queryExtension({
        msg: {
          balances: {},
        },
      })) as any;

    return remoteHeight > controlHeight;
  }, 50_000);

  const waitForDelegations = waitFor(async () => {
    const { remote_height: remoteHeight } =
      (await puppeteerContractClient.queryExtension({
        msg: {
          delegations: {},
        },
      })) as any;
    return remoteHeight > controlHeight;
  }, 50_000);

  await Promise.all([waitForBalances, waitForDelegations]);
};
