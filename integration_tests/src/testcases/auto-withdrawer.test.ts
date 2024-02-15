import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  LidoAutoWithdrawer,
  LidoCore,
  LidoFactory,
  LidoWithdrawalManager,
  LidoWithdrawalVoucher,
} from '../generated/contractLib';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
  SigningStargateClient,
} from '@cosmjs/stargate';
import { MsgTransfer } from 'cosmjs-types/ibc/applications/transfer/v1/tx';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';

const LidoFactoryClass = LidoFactory.Client;
const LidoCoreClass = LidoCore.Client;
const LidoWithdrawalVoucherClass = LidoWithdrawalVoucher.Client;
const LidoWithdrawalManagerClass = LidoWithdrawalManager.Client;
const LidoAutoWithdrawerClass = LidoAutoWithdrawer.Client;

describe('Auto withdrawer', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    contractClient?: InstanceType<typeof LidoFactoryClass>;
    coreContractClient?: InstanceType<typeof LidoCoreClass>;
    withdrawalVoucherContractClient?: InstanceType<
      typeof LidoWithdrawalVoucherClass
    >;
    withdrawalManagerContractClient?: InstanceType<
      typeof LidoWithdrawalManagerClass
    >;
    autoWithdrawerContractClient?: InstanceType<typeof LidoAutoWithdrawerClass>;
    account?: AccountData;
    icaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    gaiaUserAddress?: string;
    gaiaQueryClient?: QueryClient & StakingExtension & BankExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    neutronUserAddress?: string;
    neutronSecondUserAddress?: string;
    validatorAddress?: string;
    secondValidatorAddress?: string;
    tokenizedDenomOnNeutron?: string;
    codeIds: {
      core?: number;
      token?: number;
      withdrawalVoucher?: number;
      withdrawalManager?: number;
      strategy?: number;
      puppeteer?: number;
      validatorsSet?: number;
      distribution?: number;
      rewardsManager?: number;
    };
    exchangeRate?: number;
    tokenContractAddress?: string;
    neutronIBCDenom?: string;
    ldDenom?: string;
  } = {
    codeIds: {},
  };

  beforeAll(async () => {
    context.park = await setupPark('autowithdrawer', ['neutron', 'gaia'], true);
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
    context.gaiaClient = await SigningStargateClient.connectWithSigner(
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
      setupBankExtension,
    );

    const secondWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demo2.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    context.neutronSecondUserAddress = (
      await secondWallet.getAccounts()
    )[0].address;
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/lido_core.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/lido_token.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_withdrawal_voucher.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalVoucher = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_withdrawal_manager.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalManager = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_strategy.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.strategy = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_distribution.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.distribution = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_validators_set.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;
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
      context.codeIds.puppeteer = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_rewards_manager.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
    }
    const res = await client.upload(
      account.address,
      fs.readFileSync(join(__dirname, '../../../artifacts/lido_factory.wasm')),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoFactory.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        code_ids: {
          core_code_id: context.codeIds.core,
          token_code_id: context.codeIds.token,
          withdrawal_voucher_code_id: context.codeIds.withdrawalVoucher,
          withdrawal_manager_code_id: context.codeIds.withdrawalManager,
          strategy_code_id: context.codeIds.strategy,
          distribution_code_id: context.codeIds.distribution,
          validators_set_code_id: context.codeIds.validatorsSet,
          puppeteer_code_id: context.codeIds.puppeteer,
          rewards_manager_code_id: context.codeIds.rewardsManager,
        },
        remote_opts: {
          connection_id: 'connection-0',
          transfer_channel_id: 'channel-0',
          port_id: 'transfer',
          denom: 'stake',
          update_period: 2,
        },
        salt: 'salt',
        subdenom: 'lido',
        token_metadata: {
          description: 'Lido token',
          display: 'lido',
          exponent: 6,
          name: 'Lido liquid staking token',
          symbol: 'LIDO',
          uri: null,
          uri_hash: null,
        },
      },
      'Lido-staking-factory',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.contractClient = new LidoFactory.Client(
      client,
      context.contractAddress,
    );
    context.gaiaUserAddress = (
      await context.gaiaWallet.getAccounts()
    )[0].address;
    context.neutronUserAddress = (
      await context.wallet.getAccounts()
    )[0].address;
  });

  it('transfer tokens to neutron', async () => {
    const { gaiaClient, gaiaUserAddress, neutronUserAddress, neutronClient } =
      context;
    const res = await gaiaClient.signAndBroadcast(
      gaiaUserAddress,
      [
        {
          typeUrl: '/ibc.applications.transfer.v1.MsgTransfer',
          value: MsgTransfer.fromPartial({
            sender: gaiaUserAddress,
            sourceChannel: 'channel-0',
            sourcePort: 'transfer',
            receiver: neutronUserAddress,
            token: { denom: 'stake', amount: '2000000' },
            timeoutTimestamp: BigInt((Date.now() + 10 * 60 * 1000) * 1e6),
            timeoutHeight: {
              revisionHeight: BigInt(0),
              revisionNumber: BigInt(0),
            },
          }),
        },
      ],
      1.5,
    );
    expect(res.transactionHash).toHaveLength(64);
    await waitFor(async () => {
      const balances =
        await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
          neutronUserAddress,
        );
      context.neutronIBCDenom = balances.data.balances.find((b) =>
        b.denom.startsWith('ibc/'),
      )?.denom;
      return balances.data.balances.length > 1;
    });
    expect(context.neutronIBCDenom).toBeTruthy();
  });
  it('init', async () => {
    const { contractClient, neutronUserAddress, neutronIBCDenom } = context;
    {
      const res = await contractClient.init(context.neutronUserAddress, {
        base_denom: context.neutronIBCDenom,
        core_params: {
          idle_min_interval: 1,
          puppeteer_timeout: 60,
          unbond_batch_switch_time: 6000,
          unbonding_safe_period: 10,
          unbonding_period: 60,
        },
      });
      expect(res.transactionHash).toHaveLength(64);
    }
    const res = await contractClient.queryState();
    context.coreContractClient = new LidoCore.Client(
      context.client,
      res.core_contract,
    );
    context.withdrawalVoucherContractClient = new LidoWithdrawalVoucher.Client(
      context.client,
      res.withdrawal_voucher_contract,
    );
    context.withdrawalManagerContractClient = new LidoWithdrawalManager.Client(
      context.client,
      res.withdrawal_manager_contract,
    );
    context.tokenContractAddress = res.token_contract;
    context.exchangeRate = parseFloat(
      await context.coreContractClient.queryExchangeRate(),
    );
    context.ldDenom = `factory/${context.tokenContractAddress}/lido`;

    {
      const res = await context.coreContractClient.bond(
        neutronUserAddress,
        {},
        1.6,
        undefined,
        [
          {
            amount: '500000',
            denom: neutronIBCDenom,
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
    }
    {
      const res = await context.coreContractClient.unbond(
        neutronUserAddress,
        1.6,
        undefined,
        [
          {
            amount: Math.floor(400_000 / context.exchangeRate).toString(),
            denom: context.ldDenom,
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
    }
  });
  it('setup auto withdrawer', async () => {
    const { client, account, ldDenom } = context;
    const res = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../../artifacts/lido_auto_withdrawer.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoAutoWithdrawer.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        core_address: context.coreContractClient.contractAddress,
        withdrawal_voucher_address:
          context.withdrawalVoucherContractClient.contractAddress,
        withdrawal_manager_address:
          context.withdrawalManagerContractClient.contractAddress,
        ld_token: ldDenom,
      },
      'Lido-auto-withdrawer',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.autoWithdrawerContractClient = new LidoAutoWithdrawer.Client(
      client,
      instantiateRes.contractAddress,
    );
  });

  // TODO: test deposit
  it('bond with ld assets', async () => {
    const { neutronUserAddress, ldDenom, autoWithdrawerContractClient } =
      context;
    const res = await autoWithdrawerContractClient.bond(
      neutronUserAddress,
      {
        with_ld_assets: {},
      },
      1.6,
      undefined,
      [
        {
          amount: String(2000),
          denom: ldDenom,
        },
        {
          amount: String(50000),
          denom: 'untrn',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);

    const bondings = await autoWithdrawerContractClient.queryBondings({
      user: neutronUserAddress,
    });
    expect(bondings).toEqual({
      bondings: [
        {
          bonder: neutronUserAddress,
          deposit: [
            {
              amount: '50000',
              denom: 'untrn',
            },
          ],
          token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
        },
      ],
      next_page_key: null,
    });
  });

  it('unbond', async () => {
    const {
      neutronUserAddress,
      autoWithdrawerContractClient,
      withdrawalVoucherContractClient,
    } = context;
    const res = await autoWithdrawerContractClient.unbond(
      neutronUserAddress,
      {
        token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
      },
      1.6,
      undefined,
      [],
    );
    expect(res.transactionHash).toHaveLength(64);

    const owner = await withdrawalVoucherContractClient.queryOwnerOf({
      token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
    });
    expect(owner.owner).toEqual(neutronUserAddress);

    const bondings = await autoWithdrawerContractClient.queryBondings({
      user: neutronUserAddress,
    });
    expect(bondings).toEqual({
      bondings: [],
      next_page_key: null,
    });
  });

  it('bond with NFT', async () => {
    const {
      neutronUserAddress,
      autoWithdrawerContractClient,
      withdrawalVoucherContractClient,
    } = context;

    {
      const res = await withdrawalVoucherContractClient.approve(
        neutronUserAddress,
        {
          spender: autoWithdrawerContractClient.contractAddress,
          token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
        },
        1.6,
        undefined,
        [],
      );
      expect(res.transactionHash).toHaveLength(64);
    }
    {
      const res = await autoWithdrawerContractClient.bond(
        neutronUserAddress,
        {
          with_n_f_t: {
            token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
          },
        },
        1.6,
        undefined,
        [{ amount: '40000', denom: 'untrn' }],
      );
      expect(res.transactionHash).toHaveLength(64);
    }

    const bondings = await autoWithdrawerContractClient.queryBondings({
      user: neutronUserAddress,
    });
    expect(bondings).toEqual({
      bondings: [
        {
          bonder: neutronUserAddress,
          deposit: [
            {
              amount: '40000',
              denom: 'untrn',
            },
          ],
          token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
        },
      ],
      next_page_key: null,
    });
  });

  // TODO: figure out how to sign this tx from neutronSecondUserAccount
  it('try to withdraw before unbonding period is over', async () => {
    const { neutronUserAddress, autoWithdrawerContractClient } = context;

    await expect(
      autoWithdrawerContractClient.withdraw(
        neutronUserAddress,
        {
          token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
        },
        1.6,
        undefined,
        [],
      ),
    ).rejects.toThrowError(/is not unbonded yet/);
  });

  it('fake unbonding period', async () => {
    const { neutronUserAddress, neutronIBCDenom } = context;
    await context.coreContractClient.fakeProcessBatch(neutronUserAddress, {
      batch_id: '0',
      unbonded_amount: '499999',
    });
    await context.client.sendTokens(
      neutronUserAddress,
      context.withdrawalManagerContractClient.contractAddress,
      [{ amount: '500000', denom: neutronIBCDenom }],
      1.6,
      undefined,
    );
  });

  it('withdraw', async () => {
    const {
      neutronUserAddress,
      neutronClient,
      neutronIBCDenom,
      autoWithdrawerContractClient,
    } = context;

    const balanceBefore = parseInt(
      (
        await neutronClient.CosmosBankV1Beta1.query.queryBalance(
          neutronUserAddress,
          { denom: neutronIBCDenom },
        )
      ).data.balance.amount,
    );

    const res = await autoWithdrawerContractClient.withdraw(
      neutronUserAddress,
      {
        token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
      },
      1.6,
      undefined,
      [],
    );
    expect(res.transactionHash).toHaveLength(64);

    const balance = await neutronClient.CosmosBankV1Beta1.query.queryBalance(
      neutronUserAddress,
      { denom: neutronIBCDenom },
    );
    expect(parseInt(balance.data.balance.amount) - balanceBefore).toBe(2000);

    const bondings = await autoWithdrawerContractClient.queryBondings({
      user: neutronUserAddress,
    });
    expect(bondings).toEqual({
      bondings: [],
      next_page_key: null,
    });
  });
});
