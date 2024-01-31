import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { LidoPuppeteerAuthz } from '../generated/contractLib';
import {
  QueryClient,
  StakingExtension,
  setupAuthzExtension,
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
  DelegationsResponse,
  Transfer,
} from '../generated/contractLib/lidoPuppeteerAuthz';

import { grantAuthzPermission } from '../helpers/utils';
import { AuthzExtension } from '@cosmjs/stargate/build/modules/authz/queries';

const PuppeteerClass = LidoPuppeteerAuthz.Client;

describe.skip('Interchain puppeteer with AuthZ support', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof PuppeteerClass>;
    account?: AccountData;
    gaiaAccount?: AccountData;
    icaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningCosmWasmClient;
    gaiaQueryClient?: QueryClient & StakingExtension & AuthzExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    firstValidatorAddress?: string;
    secondValidatorAddress?: string;
    tokenizedDenom?: string;
  } = {};

  beforeAll(async () => {
    context.park = await setupPark('puppeteerauthz', ['neutron', 'gaia'], true);
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

    context.gaiaAccount = (await context.gaiaWallet.getAccounts())[0];
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
      setupAuthzExtension,
    );
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account, gaiaAccount } = context;
    const res = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../../artifacts/lido_puppeteer_authz.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoPuppeteerAuthz.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        connection_id: 'connection-0',
        port_id: 'transfer',
        update_period: 10,
        remote_denom: 'stake',
        owner: account.address,
        proxy_address: gaiaAccount.address,
      },
      'label',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new LidoPuppeteerAuthz.Client(
      client,
      context.contractAddress,
    );
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
      const res = await contractClient.queryState();
      ica = res.ica;
      return !!res.ica;
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
    let txs: Transfer[] = [];
    await waitFor(async () => {
      try {
        const res = await context.contractClient.queryTransactions();
        txs = res;
        return res.length > 0;
      } catch (e) {
        return false;
      }
    });
    expect(txs).toEqual([
      {
        amount: '10000000',
        denom: 'stake',
        recipient: context.icaAddress,
        sender: (await context.gaiaWallet.getAccounts())[0].address,
      },
    ]);
  });

  it('grant authz permissions for MsgDelegate', async () => {
    const { gaiaClient, gaiaAccount } = context;

    const msgTypeUrl = '/cosmos.staking.v1beta1.MsgDelegate';

    const result = await grantAuthzPermission(
      msgTypeUrl,
      gaiaClient,
      gaiaAccount,
      context.icaAddress,
    );

    expect(result.transactionHash).toBeTruthy();

    const res = await context.gaiaQueryClient.authz.grants(
      gaiaAccount.address,
      context.icaAddress,
      msgTypeUrl,
    );

    expect(res.grants).toHaveLength(1);
  });

  it('delegate tokens on gaia side', async () => {
    const { contractClient, account } = context;
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
    const res = await contractClient.delegate(
      account.address,
      {
        validator: context.firstValidatorAddress,
        amount: '100000',
        timeout: 6000,
      },
      1.5,
      undefined,
      [{ amount: '100000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done delegations', async () => {
    const { contractClient } = context;
    let res: Transaction[] = [];
    await waitFor(async () => {
      res = await contractClient.queryInterchainTransactions();
      return res.length > 0;
    }, 60_000);
    expect(res).toEqual([
      {
        delegate: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
          denom: 'stake',
          amount: '100000',
        },
      },
    ]);
  });

  it('grant authz permissions for MsgUndelegate', async () => {
    const { gaiaClient, gaiaAccount } = context;

    const msgTypeUrl = '/cosmos.staking.v1beta1.MsgUndelegate';

    const result = await grantAuthzPermission(
      msgTypeUrl,
      gaiaClient,
      gaiaAccount,
      context.icaAddress,
    );

    expect(result.transactionHash).toBeTruthy();

    const res = await context.gaiaQueryClient.authz.grants(
      gaiaAccount.address,
      context.icaAddress,
      msgTypeUrl,
    );
    expect(res.grants.length).toEqual(1);
  });

  it('undelegate tokens on gaia side', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.undelegate(
      account.address,
      {
        validator: context.firstValidatorAddress,
        amount: '1000',
        timeout: 600,
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done undelegation', async () => {
    const { contractClient } = context;
    let res: Transaction[] = [];
    await waitFor(async () => {
      res = await contractClient.queryInterchainTransactions();
      return res.length > 1;
    }, 20_000);
    expect(res).toEqual([
      {
        delegate: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
          denom: 'stake',
          amount: '100000',
        },
      },
      {
        undelegate: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
          denom: 'stake',
          amount: '1000',
        },
      },
    ]);
  });

  it('grant authz permissions for MsgBeginRedelegate', async () => {
    const { gaiaClient, gaiaAccount } = context;

    const msgTypeUrl = '/cosmos.staking.v1beta1.MsgBeginRedelegate';

    const result = await grantAuthzPermission(
      msgTypeUrl,
      gaiaClient,
      gaiaAccount,
      context.icaAddress,
    );

    expect(result.transactionHash).toBeTruthy();

    await context.gaiaQueryClient.authz.grants(
      gaiaAccount.address,
      context.icaAddress,
      msgTypeUrl,
    );
  });

  it('redelegate tokens on gaia side', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.redelegate(
      account.address,
      {
        validator_from: context.firstValidatorAddress,
        validator_to: context.secondValidatorAddress,
        amount: '10000',
        timeout: 600,
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done redelegation', async () => {
    const { contractClient } = context;
    let res: Transaction[] = [];
    await waitFor(async () => {
      res = await contractClient.queryInterchainTransactions();
      return res.length > 2;
    }, 40_000);
    expect(res).toEqual([
      {
        delegate: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
          denom: 'stake',
          amount: '100000',
        },
      },
      {
        undelegate: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
          denom: 'stake',
          amount: '1000',
        },
      },
      {
        redelegate: {
          interchain_account_id: 'LIDO',
          validator_from: context.firstValidatorAddress,
          validator_to: context.secondValidatorAddress,
          denom: 'stake',
          amount: '10000',
        },
      },
    ]);
  });

  it('grant authz permissions for MsgWithdrawDelegatorReward', async () => {
    const { gaiaClient, gaiaAccount } = context;

    const msgTypeUrl =
      '/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward';

    const result = await grantAuthzPermission(
      msgTypeUrl,
      gaiaClient,
      gaiaAccount,
      context.icaAddress,
    );

    expect(result.transactionHash).toBeTruthy();

    await context.gaiaQueryClient.authz.grants(
      gaiaAccount.address,
      context.icaAddress,
      msgTypeUrl,
    );
  });

  it('withdraw tokens reward on gaia side', async () => {
    const { contractClient, account } = context;
    const res = await contractClient.withdrawReward(
      account.address,
      {
        validator: context.firstValidatorAddress,
        timeout: 600,
      },
      1.5,
    );
    expect(res.transactionHash).toBeTruthy();
  });

  it('query done withdraw rewards', async () => {
    const { contractClient } = context;
    let res: Transaction[] = [];
    await waitFor(async () => {
      res = await contractClient.queryInterchainTransactions();
      return res.length > 3;
    }, 40_000);
    expect(res).toEqual([
      {
        delegate: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
          denom: 'stake',
          amount: '100000',
        },
      },
      {
        undelegate: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
          denom: 'stake',
          amount: '1000',
        },
      },
      {
        redelegate: {
          interchain_account_id: 'LIDO',
          validator_from: context.firstValidatorAddress,
          validator_to: context.secondValidatorAddress,
          denom: 'stake',
          amount: '10000',
        },
      },
      {
        withdraw_reward: {
          interchain_account_id: 'LIDO',
          validator: context.firstValidatorAddress,
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
    let delegations: DelegationsResponse;
    const { gaiaAccount } = context;

    await waitFor(async () => {
      delegations = await context.contractClient.queryDelegations();
      return delegations.delegations.length > 0;
    });
    delegations.delegations.sort((a, b) =>
      a.validator.localeCompare(b.validator),
    );
    const expected = [
      {
        delegator: gaiaAccount.address,
        validator: context.firstValidatorAddress,
        amount: {
          denom: 'stake',
          amount: '89000',
        },
      },
      {
        delegator: gaiaAccount.address,
        validator: context.secondValidatorAddress,
        amount: {
          denom: 'stake',
          amount: '10000',
        },
      },
    ];
    expected.sort((a, b) => a.validator.localeCompare(b.validator)); //fml
    expect(delegations).toMatchObject<DelegationsResponse>({
      delegations: expected,
      last_updated_height: expect.any(Number),
    });
  });
});
