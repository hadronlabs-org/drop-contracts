import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { DropPuppeteer, DropHookTester } from 'drop-ts-client';
import {
  QueryClient,
  StakingExtension,
  setupStakingExtension,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { stringToPath } from '@cosmjs/crypto';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import {
  ResponseAnswer,
  ResponseHookErrorMsg,
  ResponseHookSuccessMsg,
} from 'drop-ts-client/lib/contractLib/dropHookTester';
import { sleep } from '../helpers/sleep';

const PuppeteerClass = DropPuppeteer.Client;
const HookTesterClass = DropHookTester.Client;
const VALIDATORS_COUNT = 5;
const VALIDATORS_ICQ_LIMIT = 2;

describe('Interchain puppeteer', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof PuppeteerClass>;
    hookContractClient?: InstanceType<typeof HookTesterClass>;
    account?: AccountData;
    gaiaAccount?: AccountData;
    icaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningCosmWasmClient;
    gaiaQueryClient?: QueryClient & StakingExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    validatorAddresses: string[];
    tokenizedDenom?: string;
  } = { validatorAddresses: [] };

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'gaia'],
      {
        gaia: {
          validators: VALIDATORS_COUNT,
        },
      },
      { neutron: true, hermes: true },
    );
    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'neutron',
      },
    );

    context.gaiaWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'cosmos',
      },
    );
    context.gaiaAccount = (await context.gaiaWallet.getAccounts())[0];
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

    context.gaiaClient = await SigningCosmWasmClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.gaia.rpc}`,
      context.gaiaWallet,
      {
        gasPrice: GasPrice.fromString('0.025stake'),
      },
    );
    const tmClient = await Tendermint34Client.connect(
      `http://127.0.0.1:${context.park.ports.gaia.rpc}`,
    );
    context.gaiaQueryClient = QueryClient.withExtensions(
      tmClient,
      setupStakingExtension,
    );
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_hook_tester.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      const instantiateRes = await DropHookTester.Client.instantiate(
        client,
        account.address,
        res.codeId,
        {},
        'label',
        'auto',
        [],
      );
      expect(instantiateRes.contractAddress).toHaveLength(66);
      context.hookContractClient = new DropHookTester.Client(
        client,
        instantiateRes.contractAddress,
      );
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      const instantiateRes = await DropPuppeteer.Client.instantiate(
        client,
        account.address,
        res.codeId,
        {
          delegations_queries_chunk_size: VALIDATORS_ICQ_LIMIT,
          sdk_version: process.env.SDK_VERSION || '0.46.0',
          connection_id: 'connection-0',
          port_id: 'transfer',
          update_period: 10,
          remote_denom: 'wrong',
          owner: account.address,
          transfer_channel_id: 'channel-0',
          timeout: 60,
          allowed_senders: [
            context.hookContractClient.contractAddress,
            account.address,
          ],
        },
        'label',
        'auto',
        [],
      );
      expect(instantiateRes.contractAddress).toHaveLength(66);
      context.contractAddress = instantiateRes.contractAddress;
      context.contractClient = new DropPuppeteer.Client(
        client,
        context.contractAddress,
      );
    }
  });

  it('query configuration data', async () => {
    const { contractClient } = context;
    const config = await contractClient.queryConfig();

    expect<typeof config>(config).toEqual({
      connection_id: 'connection-0',
      delegations_queries_chunk_size: VALIDATORS_ICQ_LIMIT,
      port_id: 'transfer',
      update_period: 10,
      remote_denom: 'wrong',
      sdk_version: '0.46.0',
      timeout: 60,
      allowed_senders: [
        context.hookContractClient.contractAddress,
        context.account.address,
      ].sort(),
      transfer_channel_id: 'channel-0',
    });
  });

  it('update configuration data', async () => {
    const { contractClient, account } = context;

    const res = await contractClient.updateConfig(
      account.address,
      {
        new_config: {
          remote_denom: 'stake',
        },
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();

    const config = await contractClient.queryConfig();

    expect(config).toEqual({
      connection_id: 'connection-0',
      port_id: 'transfer',
      update_period: 10,
      remote_denom: 'stake',
      sdk_version: '0.46.0',
      timeout: 60,
      delegations_queries_chunk_size: VALIDATORS_ICQ_LIMIT,
      allowed_senders: [
        context.hookContractClient.contractAddress,
        account.address,
      ].sort(),
      transfer_channel_id: 'channel-0',
    });
  });

  it('set puppeteer address into hook tester', async () => {
    const res = await context.hookContractClient.setConfig(
      context.account.address,
      {
        puppeteer_addr: context.contractClient.contractAddress,
      },
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('register ICA', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.registerICA(
      account.address,
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
    let ica = '';
    await waitFor(async () => {
      const res = await contractClient.queryIca();
      switch (res) {
        case 'none':
        case 'in_progress':
        case 'timeout':
          return false;
        default:
          ica = res.registered.ica_address;
          return true;
      }
    }, 50_000);
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.icaAddress = ica;
  });

  it('register interchain transaction query', async () => {
    const { account, contractClient } = context;
    const res = await contractClient.registerQuery(
      account.address,
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query interchain transaction query', async () => {
    const res =
      await context.neutronClient.NeutronInterchainqueries.query.queryRegisteredQueries(
        {
          owners: [context.contractAddress],
        },
      );
    expect(res.data.registered_queries).toHaveLength(1);
    expect(res.data.registered_queries[0].owner).toEqual(
      context.contractAddress,
    );
    expect(
      JSON.parse(res.data.registered_queries[0].transactions_filter),
    ).toEqual([
      {
        field: 'transfer.recipient',
        op: 'Eq',
        value: context.icaAddress,
      },
    ]);
  });

  it('send tokens in gaia to ICA', async () => {
    const res = await context.gaiaClient.sendTokens(
      (await context.gaiaWallet.getAccounts())[0].address,
      context.icaAddress,
      [
        {
          amount: '10000000',
          denom: 'stake',
        },
      ],
      1.6,
    );
    expect(res.code).toEqual(0);
  });

  it('query received transactions on neutron side', async () => {
    let txs = [];
    await waitFor(async () => {
      try {
        const res = await context.contractClient.queryTransactions();
        txs = res;
        return res.length > 0;
      } catch (e) {
        return false;
      }
    }, 60_000);
    expect(txs).toEqual([
      {
        amount: '10000000',
        denom: 'stake',
        recipient: context.icaAddress,
        sender: (await context.gaiaWallet.getAccounts())[0].address,
      },
    ]);
  });

  it('delegate tokens on gaia side', async () => {
    const { hookContractClient, account } = context;
    for (let i = 0; i < VALIDATORS_COUNT; i++) {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath(`m/44'/118'/${i + 1}'/0/0`) as any],
        },
      );
      context.validatorAddresses.push((await wallet.getAccounts())[0].address);
    }

    const res = await hookContractClient.delegate(
      account.address,
      {
        validator: context.validatorAddresses[0],
        amount: '100000',
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('try delegate before ack is received', async () => {
    const { hookContractClient, account } = context;
    await expect(
      hookContractClient.delegate(
        account.address,
        {
          validator: context.validatorAddresses[0],
          amount: '100000',
        },
        1.5,
        undefined,
        [{ amount: '1000000', denom: 'untrn' }],
      ),
    ).to.rejects.toThrowError('txState is not equal to expected: Idle');
  });

  it('query done delegations', async () => {
    const { hookContractClient } = context;
    let res: ResponseHookSuccessMsg[] = [];
    await waitFor(async () => {
      res = await hookContractClient.queryAnswers();
      return res.length > 0;
    }, 60_000);
    expect(res.length).toEqual(1);
    expect<ResponseAnswer[]>(res[0].answers).toEqual([
      {
        delegate_response: {},
      },
    ]);
  });

  it('undelegate tokens on gaia side', async () => {
    const { hookContractClient, account } = context;
    const res = await hookContractClient.undelegate(
      account.address,
      {
        validator: context.validatorAddresses[0],
        amount: '1000',
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done undelegation', async () => {
    const { hookContractClient } = context;
    let res: ResponseHookSuccessMsg[] = [];
    await waitFor(async () => {
      res = await hookContractClient.queryAnswers();
      return res.length > 1;
    }, 20_000);
    expect(res.length).toEqual(2);
    expect(res[1].answers).toMatchObject([
      {
        undelegate_response: {
          completion_time: {
            nanos: expect.any(Number),
            seconds: expect.any(Number),
          },
        },
      },
    ]);
  });

  it('redelegate tokens on gaia side', async () => {
    const { hookContractClient, account } = context;
    const res = await hookContractClient.redelegate(
      account.address,
      {
        validator_from: context.validatorAddresses[0],
        validator_to: context.validatorAddresses[1],
        amount: '10000',
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done redelegation', async () => {
    const { hookContractClient } = context;
    let res: ResponseHookSuccessMsg[] = [];
    await waitFor(async () => {
      res = await hookContractClient.queryAnswers();
      return res.length > 2;
    }, 40_000);
    expect(res.length).toEqual(3);
    expect(res[2].answers).toEqual([
      {
        begin_redelegate_response: {
          completion_time: {
            nanos: expect.any(Number),
            seconds: expect.any(Number),
          },
        },
      },
    ]);
  });

  it('tokenize share on gaia side', async () => {
    const { hookContractClient, account } = context;
    const res = await hookContractClient.tokenizeShare(
      account.address,
      {
        validator: context.validatorAddresses[0],
        amount: '5000',
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done tokenize share', async () => {
    const { hookContractClient } = context;
    let res: ResponseHookSuccessMsg[] = [];
    await waitFor(async () => {
      res = await hookContractClient.queryAnswers();
      return res.length > 3;
    }, 40_000);
    expect(res.length).toEqual(4);
    expect(res[3].answers).toEqual([
      {
        tokenize_shares_response: {
          amount: {
            amount: '5000',
            denom: `${context.validatorAddresses[0]}/1`,
          },
        },
      },
    ]);
  });

  it('redeem share', async () => {
    const { hookContractClient, account } = context;
    const res = await hookContractClient.redeemShare(
      account.address,
      {
        validator: context.validatorAddresses[0],
        amount: '5000',
        denom: `${context.validatorAddresses[0]}/1`,
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done redeem share', async () => {
    const { hookContractClient } = context;
    let res: ResponseHookSuccessMsg[] = [];
    await waitFor(async () => {
      res = await hookContractClient.queryAnswers();
      return res.length > 4;
    }, 40_000);
    expect(res.length).toEqual(5);
    expect(res[4].answers).toEqual([
      {
        redeem_tokensfor_shares_response: {
          amount: {
            amount: '5000',
            denom: 'stake',
          },
        },
      },
    ]);
  });

  it('register balance and delegations query', async () => {
    const { contractClient, account } = context;
    const res =
      await contractClient.registerBalanceAndDelegatorDelegationsQuery(
        account.address,
        {
          validators: context.validatorAddresses,
        },
        1.5,
        undefined,
        [
          {
            amount:
              Math.ceil(VALIDATORS_COUNT / VALIDATORS_ICQ_LIMIT) * 1000000 + '',
            denom: 'untrn',
          },
        ],
      );
    expect(res.transactionHash).toBeTruthy();
  });

  it('register unbonding delegations query', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.registerDelegatorUnbondingDelegationsQuery(
      account.address,
      {
        validators: context.validatorAddresses,
      },
      1.5,
      undefined,
      [{ amount: 1000000 * VALIDATORS_COUNT + '', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });
  it('query delegations query', async () => {
    let delegations = [];
    let height = 0;
    await waitFor(async () => {
      const { delegations: d, remote_height: h } =
        (await context.contractClient.queryExtension({
          msg: { delegations: {} },
        })) as unknown as any;
      delegations = d.delegations;
      height = h;
      return d.delegations.length > 0;
    }, 60_000);
    expect(height).toBeGreaterThan(0);
    delegations.sort((a, b) => a.validator.localeCompare(b.validator));
    const expected = [
      {
        delegator: context.icaAddress,
        validator: context.validatorAddresses[0],
        amount: {
          denom: 'stake',
          amount: '89000',
        },
      },
      {
        delegator: context.icaAddress,
        validator: context.validatorAddresses[1],
        amount: {
          denom: 'stake',
          amount: '10000',
        },
      },
    ];
    expected.sort((a, b) => a.validator.localeCompare(b.validator)); //fml
    expect(delegations).toMatchObject(expected);
  });

  it('query unbonding delegations query', async () => {
    await awaitBlocks(`http://127.0.0.1:${context.park.ports.gaia.rpc}`, 4);
    const unbonding_delegations = (await context.contractClient.queryExtension({
      msg: { unbonding_delegations: {} },
    })) as unknown as any[];
    unbonding_delegations.sort((a, b) => a.query_id - b.query_id);
    const startQueryIdIndex =
      Math.ceil(VALIDATORS_COUNT / VALIDATORS_ICQ_LIMIT) + 1;

    expect(unbonding_delegations[0]).toMatchObject({
      validator_address: context.validatorAddresses[0],
      query_id: startQueryIdIndex + 1,
      unbonding_delegations: [
        {
          balance: '1000',
        },
      ],
    });
    expect(unbonding_delegations).toHaveLength(VALIDATORS_COUNT);
  });

  it('query kv keys for query_id', async () => {
    const queryIds = await context.contractClient.queryKVQueryIds();

    expect(queryIds).toEqual([
      [2, 'delegations_and_balance'],
      [3, 'delegations_and_balance'],
      [4, 'delegations_and_balance'],
      [5, 'unbonding_delegations'],
      [6, 'unbonding_delegations'],
      [7, 'unbonding_delegations'],
      [8, 'unbonding_delegations'],
      [9, 'unbonding_delegations'],
    ]);
  });

  it('grant access to some account', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.grantDelegate(
      account.address,
      {
        grantee: (await context.gaiaWallet.getAccounts())[0].address,
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();
  });
  it('wait for grant to be processed', async () => {
    await waitFor(async () => {
      const res = await context.contractClient.queryTxState();
      return res.status === 'idle';
    }, 100_000);
    const res = await context.park.executeInNetwork(
      'gaia',
      `${context.park.config.networks['gaia'].binary} query authz grants-by-grantee ${context.gaiaAccount.address} --output json`,
    );
    const out = JSON.parse(res.out);
    expect(out.grants).toHaveLength(1);
  });
  it('send a failing delegation', async () => {
    const { hookContractClient, account } = context;

    const res = await hookContractClient.delegate(
      account.address,
      {
        validator: context.validatorAddresses[0],
        amount: '10000000000000',
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });
  it('query error', async () => {
    const { hookContractClient } = context;
    let res: ResponseHookErrorMsg[] = [];
    await waitFor(async () => {
      res = await hookContractClient.queryErrors();
      return res.length > 0;
    }, 40_000);
    expect(res.length).toEqual(1);
    expect(res[0].details).toMatch(
      /ABCI code: 107: error handling packet: see events for details/,
    );
  });
  it('send with timeout', async () => {
    const { hookContractClient, account, contractClient } = context;
    await context.park.restartRelayer('hermes', 0);
    await contractClient.updateConfig(account.address, {
      new_config: { timeout: 1 },
    });
    const res = await hookContractClient.delegate(
      account.address,
      {
        validator: context.validatorAddresses[0],
        amount: '1000',
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });
  it('query timeouted error', async () => {
    const { hookContractClient } = context;
    let res: ResponseHookErrorMsg[] = [];
    await context.park.restartRelayer('hermes', 0);
    await waitFor(async () => {
      res = await hookContractClient.queryErrors();
      return res.length > 1;
    }, 160_000);
    expect(res.length).toEqual(2);
    expect(res[1].details).toEqual('Timeout');
  });
  it('ensure ICA is closed', async () => {
    const { contractClient } = context;
    const res = await contractClient.queryIca();
    expect(res).toEqual('timeout');
  });
  it('reopen ICA', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.registerICA(
      account.address,
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
    let ica = '';
    await waitFor(async () => {
      const res = await contractClient.queryIca();
      switch (res) {
        case 'none':
        case 'in_progress':
        case 'timeout':
          return false;
        default:
          ica = res.registered.ica_address;
          return true;
      }
    }, 50_000);
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.icaAddress = ica;
  });

  it('register balance and delegations query', async () => {
    const { contractClient, account } = context;
    const res =
      await contractClient.registerBalanceAndDelegatorDelegationsQuery(
        account.address,
        {
          validators: [context.validatorAddresses[1]],
        },
        1.5,
        undefined,
        [
          {
            amount:
              Math.ceil(VALIDATORS_COUNT / VALIDATORS_ICQ_LIMIT) * 1000000 + '',
            denom: 'untrn',
          },
        ],
      );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query delegations query', async () => {
    let delegations = [];
    const { delegations: d } = (await context.contractClient.queryExtension({
      msg: { delegations: {} },
    })) as unknown as any;
    delegations = d.delegations;
    expect(delegations).toHaveLength(2);
    await sleep(10_000);
    await context.park.restartRelayer('neutron', 1);
    let height = 0;
    await waitFor(async () => {
      const { delegations: d, remote_height: h } =
        (await context.contractClient.queryExtension({
          msg: { delegations: {} },
        })) as unknown as any;
      delegations = d.delegations;
      height = h;
      return d.delegations.length == 1;
    }, 80_000);
    expect(height).toBeGreaterThan(0);
    delegations.sort((a, b) => a.validator.localeCompare(b.validator));
    const expected = [
      {
        delegator: context.icaAddress,
        validator: context.validatorAddresses[1],
        amount: {
          denom: 'stake',
          amount: '10000',
        },
      },
    ];
    expected.sort((a, b) => a.validator.localeCompare(b.validator));
    expect(delegations).toMatchObject(expected);
  });
});
