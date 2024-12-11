import { afterAll, beforeAll, it, describe, expect } from 'vitest';
import Cosmopark from '@neutron-org/cosmopark';
import { setupPark } from '../testSuite';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { join } from 'path';
import fs from 'fs';
import {
  DropLazyStaking,
  DropTemplateCore,
  DropTemplateToken,
  DropTemplateFactory,
} from 'drop-ts-client';

const DropLazyStakingClient = DropLazyStaking.Client;
const DropTemplateCoreClient = DropTemplateCore.Client;
const DropTemplateTokenClient = DropTemplateToken.Client;
const DropTemplateFactoryClient = DropTemplateFactory.Client;

describe('Lazy Staking', () => {
  const context: {
    park?: Cosmopark;
    wallet?: DirectSecp256k1HdWallet;
    account?: AccountData;
    neutronClient?: InstanceType<typeof NeutronClient>;
    client?: SigningCosmWasmClient;

    lazyStakingClient?: InstanceType<typeof DropLazyStakingClient>;
    coreContractClient?: InstanceType<typeof DropTemplateCoreClient>;
    tokenContractClient?: InstanceType<typeof DropTemplateTokenClient>;
    factoryContractClient?: InstanceType<typeof DropTemplateFactoryClient>;

    dntrnDenom?: string;
  } = {};

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron'],
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
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    {
      const { codeId } = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_template_core.wasm'),
        ),
        1.5,
      );
      const { contractAddress } = await DropTemplateCoreClient.instantiate(
        client,
        account.address,
        codeId,
        {},
        'label',
        'auto',
        [],
      );
      context.coreContractClient = new DropTemplateCoreClient(
        client,
        contractAddress,
      );
      await context.coreContractClient.updateExchangeRate(account.address, {
        exchange_rate: '1',
      });
    }
    {
      const { codeId } = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_template_factory.wasm'),
        ),
        1.5,
      );
      const { contractAddress } = await DropTemplateFactoryClient.instantiate(
        client,
        account.address,
        codeId,
        {},
        'label',
        'auto',
        [],
      );
      context.factoryContractClient = new DropTemplateFactoryClient(
        client,
        contractAddress,
      );
      await context.factoryContractClient.updateState(account.address, {
        state: {
          core_contract: context.coreContractClient.contractAddress,
          distribution_contract: 'distribution_contract',
          lsm_share_bond_provider_contract: 'lsm_share_bond_provider_contract',
          native_bond_provider_contract: 'native_bond_provider_contract',
          puppeteer_contract: 'puppeteer_contract',
          rewards_manager_contract: 'rewards_manager_contract',
          rewards_pump_contract: 'rewards_pump_contract',
          splitter_contract: 'splitter_contract',
          strategy_contract: 'strategy_contract',
          token_contract: 'token_contract',
          validators_set_contract: 'validators_set_contract',
          withdrawal_manager_contract: 'withdrawal_manager_contract',
          withdrawal_voucher_contract: 'withdrawal_voucher_contract',
        },
      });
    }
    {
      const { codeId } = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_template_token.wasm'),
        ),
        1.5,
      );
      const { contractAddress } = await DropTemplateTokenClient.instantiate(
        client,
        account.address,
        codeId,
        {
          exponent: 6,
          subdenom: 'dNTRN',
          token_metadata: {
            description: 'description',
            denom_units: [],
            base: '6',
            display: 'dNTRN',
            name: 'dNTRN',
            symbol: 'dNTRN',
            uri: '',
            uri_hash: '',
          },
        },
        'label',
        'auto',
        [],
      );
      context.tokenContractClient = new DropTemplateTokenClient(
        client,
        contractAddress,
      );
      context.dntrnDenom = await context.tokenContractClient.queryDenom();
      await context.tokenContractClient.mint(account.address, {
        amount: '1000000',
      });
    }
    {
      const { codeId } = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_lazy_staking.wasm'),
        ),
        1.5,
      );
      const { contractAddress } = await DropLazyStakingClient.instantiate(
        client,
        account.address,
        codeId,
        {
          exponent: 6,
          subdenom: 'ldNTRN',
          token_metadata: {
            description: 'lazy derivative for Drop derivative for Neutron',
            denom_units: [],
            base: '6',
            display: 'ldNTRN',
            name: 'ldNTRN',
            symbol: 'ldNTRN',
            uri: '',
            uri_hash: '',
          },
          config: {
            factory_addr: context.factoryContractClient.contractAddress,
            base_denom: context.dntrnDenom,
            rewards_receiver: account.address,
          },
        },
        'label',
        'auto',
        [],
      );
      context.lazyStakingClient = new DropLazyStakingClient(
        client,
        contractAddress,
      );
    }
  });
});
