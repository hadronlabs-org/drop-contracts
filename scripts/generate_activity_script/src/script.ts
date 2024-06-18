import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SigningStargateClient } from "@cosmjs/stargate";
import { AccountData, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice, Coin } from "@cosmjs/stargate";
import { Client as DropCoreClient } from "../../../integration_tests/src/generated/contractLib/dropCore";
import { Client as DropWithdrawalManager } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalManager";
import { Client as DropWithdrawalVoucher } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalVoucher";

const CORE_CONTRACT: string = process.env.CORE_CONTRACT;

const MNEMONIC: string = process.env.MNEMONIC;
const BASE_DENOM: string = process.env.BASE_DENOM;
const FACTORY_DENOM: string = process.env.FACTORY_DENOM;
const NEUTRON_NODE_ADDRESS: string = process.env.NEUTRON_NODE_ADDRESS;

const BOND_PROB: number = Number(process.env.BOND_PROB);
const UNBOND_PROB: number = Number(process.env.UNBOND_PROB);
const WITHDRAW_PROB: number = Number(process.env.WITHDRAW_PROB);

/*
 * Each of given probabilities should be in interval [0, 1]
 * According to given probabilities relevant actions will be choosed with appropriate probability
 */
if ((0 <= BOND_PROB && BOND_PROB <= 1) === false) {
  console.error(`0 <= BOND_PROB(${BOND_PROB}) <= 1 != true`);
  process.exit(1);
}
if ((0 <= UNBOND_PROB && UNBOND_PROB <= 1) === false) {
  console.error(`0 <= UNBOND_PROB(${UNBOND_PROB}) <= 1 != true`);
  process.exit(1);
}
if ((0 <= WITHDRAW_PROB && WITHDRAW_PROB <= 1) === false) {
  console.error(`0 <= WITHDRAW_PROB(${WITHDRAW_PROB}) <= 1 != true`);
  process.exit(1);
}

enum MODE {
  BOND = "BOND",
  UNBOND = "UNBOND",
  WITHDRAW = "SEND_NFT",
}

type Wallet = {
  mainWallet?: DirectSecp256k1HdWallet;
  clientCW?: SigningCosmWasmClient;
  clientSG?: SigningStargateClient;
  mainAccounts?: readonly AccountData[];
};

type Action = {
  mode: MODE;
  txHash: string;
};

async function bond(
  drop_instance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<Action> {
  return {
    mode: MODE.BOND,
    txHash: (
      await drop_instance.bond(address, {}, "auto", "", [
        {
          amount: fund.amount,
          denom: fund.denom,
        },
      ])
    ).transactionHash,
  };
}

/* Function returns ActionLog if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function bond_random_amount(
  neutronWallet: Wallet,
  drop_instance: DropCoreClient
): Promise<Action | null> {
  const address: string = neutronWallet.mainAccounts[0].address;

  /* If here is nothing to bond on our balance, then just return null
   * Other random method will be tried to call then
   */
  let ibc_denom_balance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    BASE_DENOM
  );
  if (Number(ibc_denom_balance.amount) === 0) {
    return null;
  }

  const config = await drop_instance.queryConfig();
  const exchange_rate: number = Math.floor(
    Number(await drop_instance.queryExchangeRate())
  );
  const min_exchange_rate: number = exchange_rate + 1;

  /* Minimum amount of funds that we can send to core contract while bonding
   * Is either lsm_min_bond_amount (which is typically set in 1) or current exchange_rate parameter
   * Here we're choosing the biggest value of these two to avoid further errors
   */
  const min: number =
    Number(config.lsm_min_bond_amount) < min_exchange_rate
      ? min_exchange_rate
      : Number(config.lsm_min_bond_amount);
  if (min > Number(ibc_denom_balance.amount)) {
    return null;
  }

  /* If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  let random_amount: number = Math.floor(
    Math.random() * (Number(ibc_denom_balance.amount) - min) + min
  );
  try {
    const res = await bond(drop_instance, address, {
      amount: String(random_amount),
      denom: BASE_DENOM,
    });
    if ((await neutronWallet.clientCW.getTx(res.txHash)).code !== 0) {
      return null;
    }
    return res;
  } catch (e) {
    return null;
  }
}

async function unbond(
  drop_instance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<Action> {
  return {
    mode: MODE.UNBOND,
    txHash: (
      await drop_instance.unbond(address, "auto", "", [
        {
          amount: fund.amount,
          denom: fund.denom,
        },
      ])
    ).transactionHash,
  };
}

/* Function returns ActionLog if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function unbond_random_amount(
  neutronWallet: Wallet,
  drop_instance: DropCoreClient
): Promise<Action | null> {
  const address: string = neutronWallet.mainAccounts[0].address;
  /* If here is nothing to bond on our balance, then just return null
   * Other random method will be tried to call then
   */
  let factory_balance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    FACTORY_DENOM
  );
  if (Number(factory_balance.amount) === 0) {
    return null;
  }

  /* If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  let random_amount: number = Math.floor(
    Math.random() * Number(factory_balance.amount) + 1
  );
  try {
    const res = await unbond(drop_instance, address, {
      amount: String(random_amount),
      denom: FACTORY_DENOM,
    });
    if ((await neutronWallet.clientCW.getTx(res.txHash)).code !== 0) {
      return null;
    }
    return res;
  } catch (e) {
    return null;
  }
}

async function send_nft(
  withdrawal_voucher: DropWithdrawalVoucher,
  withdrawal_manager: DropWithdrawalManager,
  address: string,
  nft_id: string
): Promise<Action> {
  /* To withdraw unbonded amount we need to send nft to the withdrawal_manager contract
   * To do that we need to call send method on withdrawal_voucher contract (which is NFT contract underhood)
   * Field msg here is encoded base64 json object { "withdraw": {} }
   */
  return {
    mode: MODE.WITHDRAW,
    txHash: (
      await withdrawal_voucher.sendNft(
        address,
        {
          contract: withdrawal_manager.contractAddress,
          token_id: nft_id,
          msg: "eyJ3aXRoZHJhdyI6e319",
        },
        "auto",
        "",
        []
      )
    ).transactionHash,
  };
}

/* Function returns ActionLog if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function withdraw_random_nft(
  neutronWallet: Wallet,
  drop_instance: DropCoreClient
): Promise<Action | null> {
  const address: string = neutronWallet.mainAccounts[0].address;
  /* Get both withdrawal_manager and withdrawal_voucher wrappers based on querying config method
   * We need them to execute send_nft method on withdrawal_voucher with withdrawal_manager as the recepient
   * To withdraw our unbonded tokens we need to work with withdrawal_voucher' send_nft method
   */
  const config = await drop_instance.queryConfig();
  const withdrawal_manager: DropWithdrawalManager = new DropWithdrawalManager(
    neutronWallet.clientCW,
    config.withdrawal_manager_contract
  );
  const withdrawal_voucher: DropWithdrawalVoucher = new DropWithdrawalVoucher(
    neutronWallet.clientCW,
    config.withdrawal_voucher_contract
  );

  /* Get all NFT tokens in 1-st loop possessed by given address
   * Get details for each gotten NFT in 2-nd loop
   * Check batch status for each batch_id, filter who hasn't withdrawed yet in 3-rd loop
   */
  let owned_nfts = (await withdrawal_voucher.queryTokens({ owner: address }))
    .tokens;
  let current_nft_list = owned_nfts;
  while (current_nft_list.length !== 0) {
    current_nft_list = (
      await withdrawal_voucher.queryTokens({
        owner: address,
        start_after: current_nft_list[current_nft_list.length - 1],
      })
    ).tokens;
    owned_nfts = owned_nfts.concat(current_nft_list);
  }
  const owned_nfts_detailed = [];
  for (const nft of owned_nfts) {
    owned_nfts_detailed.push({
      nft_id: nft,
      details: {
        ...(await withdrawal_voucher.queryNftInfo({
          token_id: nft,
        })),
      },
    });
  }
  const withdrawn_nfts = [];
  for (const nft_detailed of owned_nfts_detailed) {
    if (
      (
        await drop_instance.queryUnbondBatch({
          batch_id: nft_detailed.details.extension.batch_id,
        })
      ).status === "withdrawn"
    ) {
      withdrawn_nfts.push(nft_detailed.nft_id);
    }
  }

  /* Nothing to withdraw
   * Return null and try another method in contract
   */
  if (withdrawn_nfts.length === 0) {
    return null;
  }

  /* Pick up random NFT from given NFT list and try to withdraw it
   * If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  const nft_id: string =
    withdrawn_nfts[Math.floor(Math.random() * withdrawn_nfts.length)];
  try {
    return await send_nft(
      withdrawal_voucher,
      withdrawal_manager,
      address,
      nft_id
    );
  } catch (e) {
    return null;
  }
}

async function main() {
  const neutronWallet: Wallet = {};
  neutronWallet.mainWallet = await DirectSecp256k1HdWallet.fromMnemonic(
    MNEMONIC,
    {
      prefix: "neutron",
    }
  );
  neutronWallet.clientCW = await SigningCosmWasmClient.connectWithSigner(
    NEUTRON_NODE_ADDRESS,
    neutronWallet.mainWallet,
    {
      gasPrice: GasPrice.fromString("0.75untrn"),
    }
  );
  neutronWallet.clientSG = await SigningStargateClient.connectWithSigner(
    NEUTRON_NODE_ADDRESS,
    neutronWallet.mainWallet,
    {
      gasPrice: GasPrice.fromString("0.75untrn"),
    }
  );
  neutronWallet.mainAccounts = await neutronWallet.mainWallet.getAccounts();

  let core_contract = new DropCoreClient(neutronWallet.clientCW, CORE_CONTRACT);
  const logs: Array<Action> = [];
  if (Math.random() <= BOND_PROB) {
    const res = await withdraw_random_nft(neutronWallet, core_contract);
    if (res !== null) {
      logs.push(res);
    }
  }
  if (Math.random() <= UNBOND_PROB) {
    const res = await unbond_random_amount(neutronWallet, core_contract);
    if (res !== null) {
      logs.push(res);
    }
  }
  if (Math.random() <= WITHDRAW_PROB) {
    const res = await bond_random_amount(neutronWallet, core_contract);
    if (res !== null) {
      logs.push(res);
    }
  }
  console.log(logs);
}

main();
