import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { LidoPuppeteer, LidoHookTester } from '../generated/contractLib';
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
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import {
  ResponseAnswer,
  ResponseHookErrorMsg,
  ResponseHookSuccessMsg,
} from '../generated/contractLib/lidoHookTester';

const PuppeteerClass = LidoPuppeteer.Client;
const HookTesterClass = LidoHookTester.Client;

describe('Interchain puppeteer', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof PuppeteerClass>;
    hookContractClient?: InstanceType<typeof HookTesterClass>;
    account?: AccountData;
    icaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningCosmWasmClient;
    gaiaQueryClient?: QueryClient & StakingExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    firstValidatorAddress?: string;
    secondValidatorAddress?: string;
    tokenizedDenom?: string;
  } = {};

  beforeAll(async () => {
    context.park = await setupPark(
      'puppeteer',
      ['neutron', 'gaia'],
      true,
      true,
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
          join(__dirname, '../../../artifacts/lido_hook_tester.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      const instantiateRes = await LidoHookTester.Client.instantiate(
        client,
        account.address,
        res.codeId,
        {},
        'label',
        [],
        'auto',
      );
      expect(instantiateRes.contractAddress).toHaveLength(66);
      context.hookContractClient = new LidoHookTester.Client(
        client,
        instantiateRes.contractAddress,
      );
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_puppeteer.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      const instantiateRes = await LidoPuppeteer.Client.instantiate(
        client,
        account.address,
        res.codeId,
        {
          connection_id: 'connection-0',
          port_id: 'transfer',
          update_period: 10,
          remote_denom: 'stake',
          owner: account.address,
          transfer_channel_id: 'channel-0',
          allowed_senders: [context.hookContractClient.contractAddress],
        },
        'label',
        [],
        'auto',
      );
      expect(instantiateRes.contractAddress).toHaveLength(66);
      context.contractAddress = instantiateRes.contractAddress;
      context.contractClient = new LidoPuppeteer.Client(
        client,
        context.contractAddress,
      );
    }
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

  it('set fees', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.setFees(
      account.address,
      {
        timeout_fee: '10000',
        ack_fee: '10000',
        recv_fee: '0',
        register_fee: '1000000',
      },
      1.5,
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
    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
        },
      );
      context.firstValidatorAddress = (await wallet.getAccounts())[0].address;
    }
    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath("m/44'/118'/2'/0/0") as any],
        },
      );
      context.secondValidatorAddress = (await wallet.getAccounts())[0].address;
    }
    const res = await hookContractClient.delegate(
      account.address,
      {
        validator: context.firstValidatorAddress,
        amount: '100000',
        timeout: 1000,
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
          validator: context.firstValidatorAddress,
          amount: '100000',
          timeout: 1000,
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
        validator: context.firstValidatorAddress,
        amount: '1000',
        timeout: 600,
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
        validator_from: context.firstValidatorAddress,
        validator_to: context.secondValidatorAddress,
        amount: '10000',
        timeout: 600,
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
        validator: context.firstValidatorAddress,
        amount: '5000',
        timeout: 600,
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
            denom: `${context.firstValidatorAddress}/1`,
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
        validator: context.firstValidatorAddress,
        amount: '5000',
        timeout: 600,
        denom: `${context.firstValidatorAddress}/1`,
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

  it('register delegations query', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.registerDelegatorDelegationsQuery(
      account.address,
      {
        validators: [
          context.firstValidatorAddress,
          context.secondValidatorAddress,
        ],
      },
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query delegations query', async () => {
    let delegations = [];
    let height = 0;
    await waitFor(async () => {
      const [d, h] = (await context.contractClient.queryExtention({
        msg: { delegations: {} },
      })) as unknown as any[];
      delegations = d.delegations;
      height = h;
      return d.delegations.length > 0;
    });
    expect(height).toBeGreaterThan(0);
    delegations.sort((a, b) => a.validator.localeCompare(b.validator));
    const expected = [
      {
        delegator: context.icaAddress,
        validator: context.firstValidatorAddress,
        amount: {
          denom: 'stake',
          amount: '89000',
        },
      },
      {
        delegator: context.icaAddress,
        validator: context.secondValidatorAddress,
        amount: {
          denom: 'stake',
          amount: '10000',
        },
      },
    ];
    expected.sort((a, b) => a.validator.localeCompare(b.validator)); //fml
    expect(delegations).toMatchObject(expected);
  });

  it('send a failing delegation', async () => {
    const { hookContractClient, account } = context;

    const res = await hookContractClient.delegate(
      account.address,
      {
        validator: context.firstValidatorAddress,
        amount: '10000000000000',
        timeout: 1000,
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
    const { hookContractClient, account } = context;
    await context.park.restartRelayer('hermes', 0);
    const res = await hookContractClient.delegate(
      account.address,
      {
        validator: context.firstValidatorAddress,
        amount: '1000',
        timeout: 1,
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
    }, 80_000);
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
});
