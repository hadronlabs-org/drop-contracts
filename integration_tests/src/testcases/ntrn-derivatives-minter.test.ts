import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropNtrnDerivativeCore,
  DropNtrnDerivativeWithdrawalVoucher,
} from 'drop-ts-client';

import { join } from 'path';
import { sleep } from '../helpers/sleep';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';

describe('Validator set', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    contracts?: {
      withdrawalVoucher?: InstanceType<
        typeof DropNtrnDerivativeWithdrawalVoucher.Client
      >;
      core?: InstanceType<typeof DropNtrnDerivativeCore.Client>;
    };
    account?: AccountData;
    client?: SigningCosmWasmClient;
    neutronClient?: InstanceType<typeof NeutronClient>;
    receiver?: string;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(t, ['neutron']);
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
    context.contracts = {};
    context.receiver = 'neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6';
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    let withdrawal_voucher_code_id: number;
    {
      const { client, account } = context;
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(
            __dirname,
            '../../../artifacts/drop_ntrn_derivative_withdrawal_voucher.wasm',
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      withdrawal_voucher_code_id = res.codeId;
    }
    {
      const { client, account } = context;
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_ntrn_derivative_core.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      const instantiateRes = await DropNtrnDerivativeCore.Client.instantiate(
        client,
        account.address,
        res.codeId,
        {
          exponent: 6,
          subdenom: 'dNTRN',
          token_metadata: {
            base: 'NTRN',
            denom_units: [],
            description: 'Drop NTRN derivative',
            display: 'dNTRN',
            name: 'Drop NTRN derivative',
            symbol: 'dNTRN',
            uri: '',
            uri_hash: '',
          },
          unbonding_period: 60,
          withdrawal_voucher_code_id: withdrawal_voucher_code_id,
        },
        'label',
        'auto',
        [],
      );
      expect(instantiateRes.contractAddress).toHaveLength(66);
      context.contractAddress = instantiateRes.contractAddress;
      context.contracts.core = new DropNtrnDerivativeCore.Client(
        client,
        context.contractAddress,
      );
      context.contracts.withdrawalVoucher =
        new DropNtrnDerivativeWithdrawalVoucher.Client(
          client,
          (await context.contracts.core.queryConfig()).withdrawal_voucher,
        );
    }
  });
  it('Try bond', async () => {
    const { contracts, account, client } = context;
    const dntrnDenom: string = await contracts.core.queryDenom();
    await contracts.core.bond(account.address, {}, undefined, undefined, [
      {
        denom: 'untrn',
        amount: '10000',
      },
    ]);
    expect((await client.getBalance(account.address, dntrnDenom)).amount).toBe(
      '10000',
    );
  });
  it('Try bond (with receiver)', async () => {
    const { contracts, account, client, receiver } = context;
    const dntrnDenom: string = await contracts.core.queryDenom();
    await contracts.core.bond(
      account.address,
      {
        receiver: receiver,
      },
      undefined,
      undefined,
      [
        {
          denom: 'untrn',
          amount: '10000',
        },
      ],
    );
    expect((await client.getBalance(receiver, dntrnDenom)).amount).toBe(
      '10000',
    );
    expect((await client.getBalance(account.address, dntrnDenom)).amount).toBe(
      '10000',
    );
  });
  it('Try unbond', async () => {
    const { contracts, account, client } = context;
    const dntrnDenom: string = await contracts.core.queryDenom();
    await contracts.core.unbond(account.address, {}, undefined, undefined, [
      {
        denom: dntrnDenom,
        amount: '5000',
      },
    ]);
    expect((await client.getBalance(account.address, dntrnDenom)).amount).toBe(
      '5000',
    );
    expect(
      (
        await contracts.withdrawalVoucher.queryTokens({
          owner: account.address,
        })
      ).tokens,
    ).toHaveLength(1);
  });
  it('Try unbond (with receiver)', async () => {
    const { contracts, account, client, receiver } = context;
    const dntrnDenom: string = await contracts.core.queryDenom();
    await contracts.core.unbond(
      account.address,
      {
        receiver: receiver,
      },
      undefined,
      undefined,
      [
        {
          denom: dntrnDenom,
          amount: '5000',
        },
      ],
    );
    expect((await client.getBalance(account.address, dntrnDenom)).amount).toBe(
      '0',
    );
    expect(
      (
        await contracts.withdrawalVoucher.queryTokens({
          owner: account.address,
        })
      ).tokens,
    ).toHaveLength(1);
    expect(
      (
        await contracts.withdrawalVoucher.queryTokens({
          owner: receiver,
        })
      ).tokens,
    ).toHaveLength(1);
  });
  it('Try withdraw', async () => {
    await sleep(60000);
    const { contracts, account } = context;
    const { tokens } = await contracts.withdrawalVoucher.queryTokens({
      owner: account.address,
    });
    const { events } = await contracts.withdrawalVoucher.sendNft(
      account.address,
      {
        contract: contracts.core.contractAddress,
        msg: 'eyJ3aXRoZHJhdyI6e319',
        token_id: tokens[0],
      },
    );
    expect(
      events
        .filter((event) => event.type == 'coin_received')
        .map((event) => event.attributes)
        .flat(),
    ).toEqual(
      expect.arrayContaining([
        {
          key: 'receiver',
          value: account.address,
        },
        {
          key: 'amount',
          value: '5000untrn',
        },
      ]),
    );
  });
  it('Try withdraw (with receiver)', async () => {
    const { contracts, account, receiver, client } = context;
    await contracts.core.bond(account.address, {}, undefined, undefined, [
      { denom: 'untrn', amount: '1000' },
    ]);
    const dntrnDenom: string = await contracts.core.queryDenom();
    await contracts.core.unbond(account.address, {}, undefined, undefined, [
      { denom: dntrnDenom, amount: '1000' },
    ]);
    await sleep(60000);
    const { tokens } = await contracts.withdrawalVoucher.queryTokens({
      owner: account.address,
    });
    const { events } = await contracts.withdrawalVoucher.sendNft(
      account.address,
      {
        contract: contracts.core.contractAddress,
        msg: 'eyJ3aXRoZHJhdyI6eyJyZWNlaXZlciI6Im5ldXRyb24xcXFxcXFxcXFxcXFxcXFxcXFxcXFxcXFxcXFxcXFxcXFodWZhYTYifX0=',
        token_id: tokens[0],
      },
    );
    expect(
      events
        .filter((event) => event.type == 'coin_received')
        .map((event) => event.attributes)
        .flat(),
    ).toEqual(
      expect.arrayContaining([
        {
          key: 'receiver',
          value: receiver,
        },
        {
          key: 'amount',
          value: '1000untrn',
        },
      ]),
    );
    expect((await client.getBalance(receiver, 'untrn')).amount).toBe('1000');
  });
});
