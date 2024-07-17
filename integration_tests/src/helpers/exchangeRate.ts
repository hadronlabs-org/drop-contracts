import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { connectComet } from '@cosmjs/tendermint-rpc';
import { QueryClient, createProtobufRpcClient } from '@cosmjs/stargate';
import {
  setupAuthExtension,
  setupBankExtension,
  setupStakingExtension,
  setupTxExtension,
} from '@cosmjs/stargate';
import { QueryClientImpl } from 'cosmjs-types/cosmos/bank/v1beta1/query';

async function calcExchangeRate(
  clientCW: SigningCosmWasmClient,
  endpoint: string,
  coreContract: string,
): Promise<number> {
  const coreConfig = await clientCW.queryContractSmart(coreContract, {
    config: {},
  });

  const queryClient = QueryClient.withExtensions(
    await connectComet(endpoint),
    setupAuthExtension,
    setupBankExtension,
    setupStakingExtension,
    setupTxExtension,
  );
  const RPC = createProtobufRpcClient(queryClient);
  const queryService = new QueryClientImpl(RPC);
  const { amount } = await queryService.SupplyOf({
    denom: coreConfig.base_denom,
  });

  const exchangeRateDenominator = Number(amount.amount);
  if (exchangeRateDenominator === 0) {
    return 1;
  }

  const delegations = await clientCW.queryContractSmart(
    coreConfig.puppeteer_contract,
    {
      extension: {
        msg: {
          delegations: {},
        },
      },
    },
  );
  const delegationsAmount: number = delegations[0].delegations.reduce(
    (acc: number, next: any) => acc + Number(next.amount.amount),
    0,
  );
  const batchID = await clientCW.queryContractSmart(coreContract, {
    current_unbond_batch: {},
  });
  const batch = await clientCW.queryContractSmart(coreContract, {
    unbond_batch: {
      batch_id: batchID,
    },
  });
  let unprocessedUnbondedAmount = 0;
  if (batch.status === 'new') {
    unprocessedUnbondedAmount += batch.expected_native_asset_amount;
  }
  if (Number(batchID) > 0) {
    const penultimate = Number(batchID) - 1;
    const penultimateBatch = await clientCW.queryContractSmart(coreContract, {
      unbond_batch: {
        batch_id: penultimate,
      },
    });
    if (penultimateBatch.status === 'unbond_requested') {
      unprocessedUnbondedAmount += batch.expected_native_asset_amount;
    }
  }
  const stakerBalance: number = Number(
    await clientCW.queryContractSmart(coreConfig.staker_contract, {
      all_balance: {},
    }),
  );
  const exchangeRateNumerator =
    delegationsAmount + stakerBalance - unprocessedUnbondedAmount;
  if (exchangeRateNumerator === 0) {
    return 1;
  }
  const exchangeRate: number = exchangeRateNumerator / exchangeRateDenominator;
  return exchangeRate;
}

export async function compareExchangeRates(
  clientCW: SigningCosmWasmClient,
  endpoint: string,
  coreContract: string,
): Promise<boolean> {
  return (
    (await calcExchangeRate(clientCW, endpoint, coreContract)).toFixed(3) ===
    (
      await clientCW.queryContractSmart(coreContract, {
        exchange_rate: {},
      })
    ).toFixed(3)
  );
}
