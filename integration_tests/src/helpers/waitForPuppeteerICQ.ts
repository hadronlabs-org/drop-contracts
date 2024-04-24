import { waitFor } from './waitFor';
import { DropCore, DropPuppeteer } from '../generated/contractLib';
import { ResponseHookSuccessMsg } from '../generated/contractLib/dropCore';

const DropCoreClass = DropCore.Client;
const DropPuppeteerClass = DropPuppeteer.Client;

export const waitForPuppeteerICQ = async (
  coreContractClient?: InstanceType<typeof DropCoreClass>,
  puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>,
): Promise<void> => {
  const puppeteerResponse = (
    (await coreContractClient.queryLastPuppeteerResponse()).response as {
      success: ResponseHookSuccessMsg;
    }
  ).success;
  const puppeteerResponseHeight = puppeteerResponse.local_height;

  return await waitFor(async () => {
    const [, lastBalanceHeight] = (await puppeteerContractClient.queryExtension(
      {
        msg: {
          balances: {},
        },
      },
    )) as any;
    return lastBalanceHeight > puppeteerResponseHeight;
  }, 50_000);
};
