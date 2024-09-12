import { afterAll, beforeAll, it, describe, expect } from 'vitest';
import Cosmopark from '@neutron-org/cosmopark';
import { setupPark } from '../testSuite';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { join } from 'path';
import fs from 'fs';
import { DropSplitter } from 'drop-ts-client';
import { sleep } from '../helpers/sleep';

const DropSplitterClass = DropSplitter.Client;

describe('Splitter', () => {
  const context: {
    park?: Cosmopark;
    wallet?: DirectSecp256k1HdWallet;
    account?: AccountData;
    neutronClient?: InstanceType<typeof NeutronClient>;
    client?: SigningCosmWasmClient;

    contractClient?: InstanceType<typeof DropSplitterClass>;
    receiver1?: string;
    receiver2?: string;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'initia'],
      {},
      {
        hermes: false,
      },
    );
    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    context.account = (await context.wallet.getAccounts())[0];
    context.neutronClient = new NeutronClient({
      apiURL: `http://127.0.0.1:${context.park.ports.neutron.rest}`,
      rpcURL: `127.0.0.1:${context.park.ports.neutron.rpc}`,
      prefix: 'neutron',
    });
    context.client = await SigningCosmWasmClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.neutron.rpc}`,
      context.wallet,
      {
        gasPrice: GasPrice.fromString('0.025untrn'),
      },
    );
    context.receiver1 = (
      await (
        await DirectSecp256k1HdWallet.generate(12, {
          prefix: 'neutron',
        })
      ).getAccounts()
    )[0].address;
    context.receiver2 = (
      await (
        await DirectSecp256k1HdWallet.generate(12, {
          prefix: 'neutron',
        })
      ).getAccounts()
    )[0].address;
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    console.log('123');
    await sleep(30_000);
  });
});
