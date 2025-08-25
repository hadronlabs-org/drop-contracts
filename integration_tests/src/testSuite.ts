import cosmopark, { CosmoparkConfig } from '@neutron-org/cosmopark';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { StargateClient } from '@cosmjs/stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { waitFor } from './helpers/waitFor';
import { sleep } from './helpers/sleep';
import fs from 'fs';
import {
  CosmoparkNetworkConfig,
  CosmoparkRelayer,
} from '@neutron-org/cosmopark/lib/types';
import { Suite } from 'vitest';
const packageJSON = require(`${__dirname}/../package.json`);
const VERSION = (process.env.CI ? '_' : ':') + packageJSON.version;
const ORG = process.env.CI ? 'neutronorg/lionco-contracts:' : '';

const keys = [
  'master',
  'hermes',
  'ibcrelayer',
  'demowallet1',
  'demowallet2',
  'neutronqueryrelayer',
  'demo1',
  'demo2',
  'demo3',
] as const;

const TIMEOUT = 10_000;

const redefinedParams =
  process.env.REMOTE_CHAIN_OPTS && fs.existsSync(process.env.REMOTE_CHAIN_OPTS)
    ? JSON.parse(fs.readFileSync(process.env.REMOTE_CHAIN_OPTS).toString())
    : {};

const networkConfigs = {
  lsm: {
    binary: redefinedParams.binary || 'gaiad',
    chain_id: 'testlsm',
    denom: redefinedParams.denom || 'stake',
    image: `${ORG}${process.env.REMOTE_CHAIN ?? 'gaia-test'}${VERSION}`,
    prefix: redefinedParams.prefix || 'cosmos',
    trace: true,
    validators: 2,
    commands: redefinedParams.commands,
    validators_balance: ['1900000000', '100000000'],
    genesis_opts: redefinedParams.genesisOpts || {
      'app_state.slashing.params.downtime_jail_duration': '10s',
      'app_state.slashing.params.signed_blocks_window': '10',
      'app_state.slashing.params.min_signed_per_window': '0.9',
      'app_state.slashing.params.slash_fraction_downtime': '0.1',
      'app_state.staking.params.validator_bond_factor': '10',
      'app_state.staking.params.unbonding_time': '1814400s',
      'app_state.mint.minter.inflation': '0.9',
      'app_state.mint.params.inflation_max': '0.95',
      'app_state.mint.params.inflation_min': '0.5',
      'app_state.interchainaccounts.host_genesis_state.params.allow_messages': [
        '*',
      ],
    },
    config_opts: {
      'rpc.laddr': 'tcp://0.0.0.0:26657',
    },
    app_opts: {
      'api.enable': true,
      'api.address': 'tcp://0.0.0.0:1317',
      'api.swagger': true,
      'grpc.enable': true,
      'grpc.address': '0.0.0.0:9090',
      'minimum-gas-prices': redefinedParams.denom
        ? `0${redefinedParams.denom}`
        : '0stake',
      'rosetta.enable': true,
    },
    upload: redefinedParams.upload || ['./artifacts/scripts/init-gaia.sh'],
    post_start: redefinedParams.postUpload || [
      `/opt/init-gaia.sh > /opt/init-gaia.log 2>&1`,
    ],
  },
  gaia: {
    binary: redefinedParams.binary || 'gaiad',
    chain_id: 'testgaia',
    denom: redefinedParams.denom || 'stake',
    image: `${ORG}${process.env.REMOTE_CHAIN ?? 'gaia-test'}${VERSION}`,
    prefix: redefinedParams.prefix || 'cosmos',
    trace: true,
    validators: 2,
    commands: redefinedParams.commands,
    validators_balance: [
      '1900000000',
      '100000000',
      '100000000',
      '100000000',
      '100000000',
    ],
    genesis_opts: redefinedParams.genesisOpts || {
      'app_state.slashing.params.downtime_jail_duration': '10s',
      'app_state.slashing.params.signed_blocks_window': '10',
      'app_state.slashing.params.min_signed_per_window': '0.9',
      'app_state.slashing.params.slash_fraction_downtime': '0.1',
      'app_state.staking.params.validator_bond_factor': '10',
      'app_state.staking.params.unbonding_time': '1814400s',
      'app_state.mint.minter.inflation': '0.9',
      'app_state.mint.params.inflation_max': '0.95',
      'app_state.mint.params.inflation_min': '0.5',
      'app_state.interchainaccounts.host_genesis_state.params.allow_messages': [
        '*',
      ],
    },
    config_opts: {
      'rpc.laddr': 'tcp://0.0.0.0:26657',
    },
    app_opts: {
      'api.enable': true,
      'api.address': 'tcp://0.0.0.0:1317',
      'api.swagger': true,
      'grpc.enable': true,
      'grpc.address': '0.0.0.0:9090',
      'minimum-gas-prices': redefinedParams.denom
        ? `0${redefinedParams.denom}`
        : '0stake',
      'rosetta.enable': true,
    },
    upload: redefinedParams.upload || ['./artifacts/scripts/init-gaia.sh'],
    post_start: redefinedParams.postUpload || [
      `/opt/init-gaia.sh > /opt/init-gaia.log 2>&1`,
    ],
  },
  neutronv2: {
    binary: 'neutrond',
    chain_id: 'ntrntest',
    denom: 'untrn',
    image: `${ORG}neutronv2-test${VERSION}`,
    prefix: 'neutron',
    loglevel: 'debug',
    trace: true,
    public: true,
    validators: 2,
    validators_balance: ['1900000000', '100000000', '100000000'],
    upload: [
      './artifacts/contracts',
      './artifacts/contracts_thirdparty',
      './artifacts/scripts/init-neutrond.sh',
    ],
    post_init: ['CHAINID=ntrntest CHAIN_DIR=/opt /opt/init-neutrond.sh'],
    genesis_opts: {
      'app_state.crisis.constant_fee.denom': 'untrn',
    },
    config_opts: {
      'consensus.timeout_commit': '500ms',
      'consensus.timeout_propose': '500ms',
    },
    app_opts: {
      'api.enable': 'true',
      'api.address': 'tcp://0.0.0.0:1317',
      'api.swagger': 'true',
      'grpc.enable': 'true',
      'grpc.address': '0.0.0.0:9090',
      'minimum-gas-prices': '0.0025untrn',
      'rosetta.enable': 'true',
      'telemetry.prometheus-retention-time': 1000,
    },
  },
  neutron: {
    binary: 'neutrond',
    chain_id: 'ntrntest',
    denom: 'untrn',
    image: `${ORG}neutron-test${VERSION}`,
    prefix: 'neutron',
    loglevel: 'debug',
    trace: true,
    public: true,
    type: 'ics',
    upload: [
      './artifacts/contracts',
      './artifacts/contracts_thirdparty',
      './artifacts/scripts/init-neutrond.sh',
    ],
    post_init: ['CHAINID=ntrntest CHAIN_DIR=/opt /opt/init-neutrond.sh'],
    genesis_opts: {
      'app_state.crisis.constant_fee.denom': 'untrn',
    },
    config_opts: {
      'consensus.timeout_commit': '500ms',
      'consensus.timeout_propose': '500ms',
    },
    app_opts: {
      'api.enable': 'true',
      'api.address': 'tcp://0.0.0.0:1317',
      'api.swagger': 'true',
      'grpc.enable': 'true',
      'grpc.address': '0.0.0.0:9090',
      'minimum-gas-prices': '0.0025untrn',
      'rosetta.enable': 'true',
      'telemetry.prometheus-retention-time': 1000,
    },
  },
};

const relayersConfig = {
  hermes: {
    balance: '1000000000',
    binary: 'hermes',
    config: {
      'chains.0.gas_multiplier': 1.2,
      'chains.0.trusting_period': '112h0m0s',
      'chains.1.gas_multiplier': 1.2,
      'chains.1.trusting_period': '168h0m0s',
    },
    image: `${ORG}hermes-test${VERSION}`,
    log_level: 'trace',
    type: 'hermes',
  },
  neutron: {
    balance: '1000000000',
    binary: 'neutron-query-relayer',
    image: `${ORG}neutron-query-relayer-test${VERSION}`,
    log_level: 'debug',
    type: 'neutron',
  },
};

type Keys = (typeof keys)[number];

const awaitFirstBlock = (rpc: string): Promise<void> =>
  waitFor(async () => {
    try {
      const controller = new AbortController();
      setTimeout(() => controller.abort(), 1000);
      await fetch(rpc, {
        method: 'GET',
        signal: controller.signal,
      });
      const client = await StargateClient.connect(rpc);
      const block = await client.getBlock();
      if (block.header.height > 1) {
        return true;
      }
    } catch (e) {
      return false;
    }
  }, 20_000);

export const awaitBlocks = async (
  rpc: string,
  blocks: number,
): Promise<void> => {
  const start = Date.now();
  const client = await StargateClient.connect(rpc);
  const initBlock = await client.getBlock();
  // eslint-disable-next-line no-constant-condition
  while (true) {
    try {
      const block = await client.getBlock();
      if (block.header.height - initBlock.header.height >= blocks) {
        break;
      }
      if (Date.now() - start > TIMEOUT) {
        throw new Error('Timeout waiting for the specific block');
      }
    } catch (e) {
      //noop
    }
    await sleep(1000);
  }
};

const awaitNeutronChannels = (rest: string, rpc: string): Promise<void> =>
  waitFor(async () => {
    try {
      const client = new NeutronClient({
        apiURL: `http://${rest}`,
        rpcURL: `http://${rpc}`,
        prefix: 'neutron',
      });
      const res = await client.IbcCoreChannelV1.query.queryChannels(undefined, {
        timeout: 1000,
      });
      console.log(res);
      if (
        res.data.channels.length > 0 &&
        res.data.channels[0].counterparty.channel_id !== ''
      ) {
        return true;
      }
      await sleep(10000);
    } catch (e) {
      await sleep(10000);
      return false;
    }
  }, 100_000);

export const generateWallets = (): Promise<Record<Keys, string>> =>
  keys.reduce(
    async (acc, key) => {
      const accObj = await acc;
      const wallet = await DirectSecp256k1HdWallet.generate(12, {
        prefix: 'neutron',
      });
      accObj[key] = wallet.mnemonic;
      return accObj;
    },
    Promise.resolve({} as Record<Keys, string>),
  );

type NetworkOptsType = Partial<Record<keyof typeof networkConfigs | '*', any>>;
const getNetworkConfig = (
  network: string,
  opts: NetworkOptsType = {},
): CosmoparkNetworkConfig => {
  let config = networkConfigs[network];
  const extOpts = opts['*'] || opts[network] || {};
  for (const [key, value] of Object.entries(extOpts)) {
    if (typeof value === 'object') {
      config = { ...config, [key]: { ...config[key], ...value } };
    } else {
      config = { ...config, [key]: value };
    }
  }
  return config;
};

type RelayerOptsType = Partial<Record<keyof typeof relayersConfig, any>>;
const getRelayerConfig = (
  relayer: string,
  opts: RelayerOptsType,
): CosmoparkRelayer => {
  relayer;
  let config = relayersConfig[relayer] || {};

  for (const [key, value] of Object.entries(opts)) {
    if (typeof value === 'object') {
      config = { ...config, [key]: { ...config[key], ...value } };
    } else {
      config = { ...config, [key]: value };
    }
  }
  return config;
};

function isSuite(t: any): t is Suite {
  return t && t.type === 'suite' && t.suite;
}

export const setupPark = async (
  t: Readonly<Suite | File>,
  networks: string[] = [],
  opts?: NetworkOptsType, // Key is path to the param, value is Record of network name and value
  relayers: Partial<Record<keyof typeof relayersConfig, any | boolean>> = {},
): Promise<cosmopark> => {
  const context = ((t: Readonly<Suite | File>) => {
    if (isSuite(t)) {
      return t.suite.file.filepath
        ?.split('/')
        .pop()!
        .split('.')[0]
        .replace(/[-_]/g, '');
    } else {
      throw new Error('Invalid context');
    }
  })(t);
  const wallets = await generateWallets();
  const config: CosmoparkConfig = {
    context,
    networks: {},
    master_mnemonic: wallets.master,
    loglevel: 'info',
    wallets: {
      demowallet1: {
        mnemonic: wallets.demowallet1,
        balance: '1000000000',
      },
      demowallet2: {
        mnemonic: wallets.demowallet2,
        balance: '1000000000',
      },
      demo1: { mnemonic: wallets.demo1, balance: '1000000000' },
      demo2: { mnemonic: wallets.demo2, balance: '1000000000' },
      demo3: { mnemonic: wallets.demo3, balance: '1000000000' },
    },
  };
  for (const network of networks) {
    config.networks[network] = getNetworkConfig(network, opts);
  }
  config.relayers = [];
  if (relayers.hermes) {
    const connections = networks.reduce((connections, network, index, all) => {
      if (index === all.length - 1) {
        return connections;
      }
      for (let i = index + 1; i < all.length; i++) {
        connections.push([network, all[i]]);
      }
      return connections;
    }, []);
    config.relayers.push({
      ...getRelayerConfig(
        'hermes',
        relayers.hermes === true ? {} : relayers.hermes,
      ),
      networks,
      connections: connections,
      mnemonic: wallets.hermes,
    } as any);
  }
  if (relayers.neutron) {
    config.relayers.push({
      ...getRelayerConfig(
        'neutron',
        relayers.neutron === true ? {} : relayers.neutron,
      ),
      networks,
      mnemonic: wallets.neutronqueryrelayer,
    } as any);
  }
  const instance = await cosmopark.create(config);
  await Promise.all(
    Object.entries(instance.ports).map(([network, ports]) =>
      awaitFirstBlock(`http://127.0.0.1:${ports.rpc}`).catch((e) => {
        console.log(`Failed to await first block for ${network}: ${e}`);
        throw e;
      }),
    ),
  );
  if (relayers.hermes) {
    await awaitNeutronChannels(
      `127.0.0.1:${instance.ports['neutronv2'].rest}`,
      `127.0.0.1:${instance.ports['neutronv2'].rpc}`,
    ).catch((e) => {
      console.log(`Failed to await neutron channels: ${e}`);
      throw e;
    });
  }
  return instance;
};
