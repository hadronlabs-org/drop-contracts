import fs from 'fs';
import path from 'path';
import { ParsedGraph } from './graphvis';
import { ExecuteResult } from '@cosmjs/cosmwasm-stargate';
import { DropCore } from '../generated/contractLib';
import { IndexedTx, StdFee } from '@cosmjs/stargate';
import { waitForTx } from './waitForTx';

const gvFile = path.resolve(__dirname, '../../../graph.gv');
const gvContent = fs.readFileSync(gvFile, 'utf-8');
const parsedTree = new ParsedGraph(gvContent);

const DropCoreClass = DropCore.Client;
type Coin = { denom: string; amount: string };

type CoreClass = InstanceType<typeof DropCoreClass>;

function returnClient() {
  return this.client;
}

export const instrumentCoreClass = (c: CoreClass) => {
  const originalTick = c.tick;
  c.tick = async (
    sender: string,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ): Promise<ExecuteResult> => {
    const res = await originalTick(sender, fee, memo, funds);
    (c as any).returnClient = returnClient.bind(c);
    await waitForTx((c as any).returnClient(), res.transactionHash);
    const tx = (await (c as any)
      .returnClient()
      .getTx(res.transactionHash)) as IndexedTx;
    const knots = tx.events
      .map((e) =>
        e.attributes.filter((a) => a.key === 'knot').map((a) => `K${a.value}`),
      )
      .flat();
    if (!parsedTree.hasPath(knots)) {
      throw new Error('Invalid Knot path ' + knots.join(' -> '));
    }
    return res;
  };
  return c;
};
