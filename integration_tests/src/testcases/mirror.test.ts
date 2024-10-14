import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { DropCore, DropFactory, DropMirror } from 'drop-ts-client';
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
import { stringToPath } from '@cosmjs/crypto';
import { instrumentCoreClass } from '../helpers/knot';
import { sleep } from '../helpers/sleep';
import { exec } from 'child_process';

const DropMirrorClass = DropMirror.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const UNBONDING_TIME = 360;

describe('Mirror', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    mirrorContractClient?: InstanceType<typeof DropMirrorClass>;
    account?: AccountData;
    icaAddress?: string;
    rewardsPumpIcaAddress?: string;
    stakerIcaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    gaiaUserAddress?: string;
    gaiaUserAddress2?: string;
    gaiaQueryClient?: QueryClient & StakingExtension & BankExtension;
    neutronRPCEndpoint?: string;
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
      staker?: number;
      validatorsSet?: number;
      distribution?: number;
      rewardsManager?: number;
      splitter?: number;
      pump?: number;
      mirror?: number;
    };
    exchangeRate?: number;
    neutronIBCDenom?: string;
    ldDenom?: string;
  } = { codeIds: {} };

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'gaia'],
      {
        gaia: {
          genesis_opts: {
            'app_state.staking.params.unbonding_time': `${UNBONDING_TIME}s`,
          },
        },
      },
      {
        neutron: true,
        hermes: {
          config: {
            'chains.1.trusting_period': '2m0s',
          },
        },
      },
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
    context.gaiaWallet2 = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demo1.mnemonic,
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
    context.neutronRPCEndpoint = `http://127.0.0.1:${context.park.ports.neutron.rpc}`;
    context.client = await SigningCosmWasmClient.connectWithSigner(
      context.neutronRPCEndpoint,
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

  it('transfer tokens to neutron', async () => {
    context.gaiaUserAddress = (
      await context.gaiaWallet.getAccounts()
    )[0].address;
    context.gaiaUserAddress2 = (
      await context.gaiaWallet2.getAccounts()
    )[0].address;
    context.neutronUserAddress = (
      await context.wallet.getAccounts()
    )[0].address;
    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
        },
      );
      context.validatorAddress = (await wallet.getAccounts())[0].address;
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
    }, 60_000);
    expect(context.neutronIBCDenom).toBeTruthy();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    context.codeIds = {};
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(join(__dirname, '../../../artifacts/drop_core.wasm')),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_token.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_withdrawal_voucher.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalVoucher = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_withdrawal_manager.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalManager = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_strategy.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.strategy = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_distribution.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.distribution = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_validators_set.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.puppeteer = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_rewards_manager.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_staker.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.staker = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_splitter.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.splitter = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(join(__dirname, '../../../artifacts/drop_pump.wasm')),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.pump = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_mirror.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.mirror = res.codeId;
    }
    const res = await client.upload(
      account.address,
      Uint8Array.from(
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_factory.wasm'),
        ),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropFactory.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        sdk_version: process.env.SDK_VERSION || '0.46.0',
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
          staker_code_id: context.codeIds.staker,
          splitter_code_id: context.codeIds.splitter,
          rewards_pump_code_id: context.codeIds.pump,
        },
        remote_opts: {
          connection_id: 'connection-0',
          transfer_channel_id: 'channel-0',
          reverse_transfer_channel_id: 'channel-0',
          port_id: 'transfer',
          denom: 'stake',
          update_period: 2,
          timeout: {
            local: 60,
            remote: 60,
          },
        },
        salt: 'salt',
        subdenom: 'drop',
        token_metadata: {
          description: 'Drop token',
          display: 'drop',
          exponent: 6,
          name: 'Drop liquid staking token',
          symbol: 'DROP',
          uri: null,
          uri_hash: null,
        },
        local_denom: 'untrn',
        base_denom: context.neutronIBCDenom,
        core_params: {
          idle_min_interval: 40,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: UNBONDING_TIME,
          lsm_redeem_threshold: 10,
          lsm_redeem_max_interval: 60_000,
          lsm_min_bond_amount: '1',
          min_stake_amount: '2',
          icq_update_delay: 5,
        },
        staker_params: {
          min_stake_amount: '10000',
          min_ibc_transfer: '10000',
        },
      },
      'drop-staking-factory',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.factoryContractClient = new DropFactory.Client(
      client,
      context.contractAddress,
    );
  });

  it('query factory state', async () => {
    const { factoryContractClient: contractClient, neutronClient } = context;
    const res = await contractClient.queryState();
    expect(res).toBeTruthy();
    const tokenContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.token_contract,
      );
    expect(tokenContractInfo.data.contract_info.label).toBe(
      'drop-staking-token',
    );
    const coreContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.core_contract,
      );
    expect(coreContractInfo.data.contract_info.label).toBe('drop-staking-core');
    const withdrawalVoucherContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.withdrawal_voucher_contract,
      );
    expect(withdrawalVoucherContractInfo.data.contract_info.label).toBe(
      'drop-staking-withdrawal-voucher',
    );
    const withdrawalManagerContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.withdrawal_manager_contract,
      );
    expect(withdrawalManagerContractInfo.data.contract_info.label).toBe(
      'drop-staking-withdrawal-manager',
    );
    const puppeteerContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.puppeteer_contract,
      );
    expect(puppeteerContractInfo.data.contract_info.label).toBe(
      'drop-staking-puppeteer',
    );
    context.coreContractClient = instrumentCoreClass(
      new DropCore.Client(context.client, res.core_contract),
    );
    context.ldDenom = `factory/${res.token_contract}/drop`;
  });

  it('instantiate mirror', async () => {
    const res = await DropMirror.Client.instantiate(
      context.client,
      context.neutronUserAddress,
      context.codeIds.mirror,
      {
        core_contract: context.coreContractClient.contractAddress,
        source_channel: 'channel-0',
        source_port: 'transfer',
        ibc_timeout: 10,
        prefix: 'cosmos',
      },
      'mirror',
      1.6,
    );
    expect(res.contractAddress).toHaveLength(66);
    context.mirrorContractClient = new DropMirror.Client(
      context.client,
      res.contractAddress,
    );
  });

  it('bond with wrong receiver', async () => {
    await expect(
      context.mirrorContractClient.bond(
        context.neutronUserAddress,
        {
          receiver: 'omfg',
        },
        1.6,
      ),
    ).to.rejects.toThrow(/Wrong receiver address/);
  });

  it('bond without funds attached', async () => {
    await expect(
      context.mirrorContractClient.bond(
        context.neutronUserAddress,
        {
          receiver: context.gaiaUserAddress,
        },
        1.6,
      ),
    ).to.rejects.toThrow(/No funds sent/);
  });
  it('bond with wrong denom', async () => {
    await expect(
      context.mirrorContractClient.bond(
        context.neutronUserAddress,
        {
          receiver: context.gaiaUserAddress,
        },
        1.6,
        '',
        [
          {
            denom: 'untrn',
            amount: '1000',
          },
        ],
      ),
    ).to.rejects.toThrow(/perhaps, this is not an IBC denom/);
  });

  it('bond', async () => {
    const res = await context.mirrorContractClient.bond(
      context.neutronUserAddress,
      {
        receiver: context.gaiaUserAddress,
      },
      1.6,
      '',
      [
        {
          denom: context.neutronIBCDenom,
          amount: '10000',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
  });

  it('verify one query', async () => {
    const res = await context.mirrorContractClient.queryOne({ id: 1 });
    expect(res).toEqual({
      amount: '10000',
      backup: null,
      id: 1,
      received: {
        amount: '10000',
        denom: context.ldDenom,
      },
      receiver: context.gaiaUserAddress,
      return_type: 'remote',
      state: 'bonded',
    });
  });
  it('verify all query', async () => {
    const res = await context.mirrorContractClient.queryAll({});
    expect(res).toEqual([
      [
        1,
        {
          amount: '10000',
          backup: null,
          id: 1,
          received: {
            amount: '10000',
            denom: context.ldDenom,
          },
          receiver: context.gaiaUserAddress,
          return_type: 'remote',
          state: 'bonded',
        },
      ],
    ]);
  });
  it('complete', async () => {
    const res = await context.mirrorContractClient.complete(
      context.neutronUserAddress,
      { items: [1] },
      1.6,
      '',
      [
        {
          denom: 'untrn',
          amount: '100000',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
  });
  it('wait for the transfer', async () => {
    await waitFor(async () => {
      const balances = await context.gaiaQueryClient.bank.allBalances(
        context.gaiaUserAddress,
      );
      return balances.length > 1;
    });
  });
  it('verify bond result', async () => {
    const balances = await context.gaiaQueryClient.bank.allBalances(
      context.gaiaUserAddress,
    );
    expect(balances.find((b) => b.denom.startsWith('ibc/'))?.amount).toEqual(
      '10000',
    );
  });
  it('verify bond is gone from the mirror contract', async () => {
    await sleep(10_000);
    await expect(
      context.mirrorContractClient.queryOne({ id: 1 }),
    ).to.rejects.toThrow(/not found/);
  });
  describe('bond with wrong receiver', () => {
    it('bond', async () => {
      const res = await context.mirrorContractClient.bond(
        context.neutronUserAddress,
        {
          receiver:
            'cosmos1yvququ0g6q2qm4arevf22nsvm6y2zmvza8pwggcpcpc735q5pht68jcj548qeh5g59kc96wzus3szckscyg',
          backup: context.neutronSecondUserAddress,
        },
        1.6,
        '',
        [
          {
            denom: context.neutronIBCDenom,
            amount: '10000',
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
      await sleep(3_000);
    });
    it('complete', async () => {
      const res = await context.mirrorContractClient.complete(
        context.neutronUserAddress,
        { items: [2] },
        1.6,
        '',
        [
          {
            denom: 'untrn',
            amount: '100000',
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
    });
    it('verify state', async () => {
      await sleep(15_000);
      const res = await context.mirrorContractClient.queryOne({ id: 2 });
      expect(res.state).toEqual('bonded');
    });
    it('switch return type', async () => {
      // we need another account here
      const secondWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.wallets.demo2.mnemonic,
        {
          prefix: 'neutron',
        },
      );
      const client = await SigningCosmWasmClient.connectWithSigner(
        context.neutronRPCEndpoint,
        secondWallet,
        {
          gasPrice: GasPrice.fromString('0.025untrn'),
        },
      );
      const mirrorContractClient = new DropMirror.Client(
        client,
        context.mirrorContractClient.contractAddress,
      );
      const res = await mirrorContractClient.changeReturnType(
        context.neutronSecondUserAddress,
        { id: 2, return_type: 'local' },
        1.6,
      );
      expect(res.transactionHash).toHaveLength(64);
    });
    it('complete', async () => {
      const res = await context.mirrorContractClient.complete(
        context.neutronUserAddress,
        { items: [2] },
        1.6,
        '',
        [
          {
            denom: 'untrn',
            amount: '100000',
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
      await sleep(2_000);
    });
    it('verify balance', async () => {
      const res =
        await context.neutronClient.CosmosBankV1Beta1.query.queryBalance(
          context.neutronSecondUserAddress,
          { denom: context.ldDenom },
        );
      expect(res.data.balance.amount).toEqual('10000');
    });
    it('verify bond is gone from the mirror contract', async () => {
      await sleep(10_000);
      await expect(
        context.mirrorContractClient.queryOne({ id: 2 }),
      ).to.rejects.toThrow(/not found/);
    });
  });
  describe('bond with timeout', () => {
    it('update timeout', async () => {
      const res = await context.mirrorContractClient.updateConfig(
        context.neutronUserAddress,
        {
          new_config: {
            ibc_timeout: 2,
          },
        },
        1.6,
      );
      expect(res.transactionHash).toHaveLength(64);
    });
    it('bond', async () => {
      const res = await context.mirrorContractClient.bond(
        context.neutronUserAddress,
        {
          receiver: context.gaiaUserAddress,
          backup: context.neutronSecondUserAddress,
        },
        1.6,
        '',
        [
          {
            denom: context.neutronIBCDenom,
            amount: '10000',
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
    });
    it('stop gaia', async () => {
      await context.park.pauseNetwork('gaia');
      await context.park.pauseRelayer('hermes', 1);
      await sleep(30_000);
    });
    it('complete', async () => {
      const res = await context.mirrorContractClient.complete(
        context.neutronUserAddress,
        { items: [3] },
        1.6,
        '',
        [
          {
            denom: 'untrn',
            amount: '100000',
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
    });

    it('verify bond state', async () => {
      exec('docker ps -a', (err, stdout, stderr) => {
        console.log({
          err,
          stdout,
          stderr,
        });
      });
      await context.park.restartRelayer('hermes', 0);
      await await sleep(60_000);
      const res = await context.mirrorContractClient.queryOne({ id: 3 });
      expect(res.state).toEqual('bonded');
    });
  });
});
